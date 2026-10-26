# Identity E2E for the SEA Forge Windows control plane, run on a real Windows
# runner. Every assertion prints a PASS or FAIL line to the log. The script
# exits non-zero if any assertion fails.
#
# Checks:
#   (c) the bound runner user's protected verb is not refused for identity
#   (d) a different local user (sfprobe) is refused: the pipe DACL denies the
#       open, or the server refuses the verb with an identity error
#   (e) a second server on the same socket refuses to start
#   (f) a process that pre-creates the derived pipe name makes the server
#       refuse to start (squatting)
#   (g) sfprobe is removed at the end, whatever happened
#
# The pipe name is derived here the same way as sea_forge_core::pipe_name_for:
# SHA-256 of the verbatim canonical socket path, first 32 hex digits. The
# derivation is pinned by a Rust unit test in sea-forge-core.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [string] $ServerExe,
    [string] $Work = 'C:\sfe2e'
)

$ErrorActionPreference = 'Stop'
$script:Fails = 0
$script:Passes = 0
$log = Join-Path $Work 'identity-e2e.log'
New-Item -ItemType Directory -Force -Path $Work | Out-Null
Set-Content -Path $log -Value '' -Encoding utf8

function Say([string] $Message) {
    $line = '[{0}] {1}' -f (Get-Date -Format 'HH:mm:ss.fff'), $Message
    Add-Content -Path $log -Value $line -Encoding utf8
    Write-Host $line
}

function Check([bool] $Ok, [string] $Name, [string] $Detail = '') {
    if ($Ok) {
        $script:Passes++
        Say "PASS: $Name $Detail".TrimEnd()
    } else {
        $script:Fails++
        Say "FAIL: $Name $Detail".TrimEnd()
    }
}

# Mirrors sea_forge_core::pipe_name_for for a socket file whose parent exists.
function Get-PipeName([string] $SocketPath) {
    $parent = Split-Path -Parent $SocketPath
    $leaf = Split-Path -Leaf $SocketPath
    $canonical = '\\?\' + (Join-Path (Resolve-Path -LiteralPath $parent).ProviderPath $leaf)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $digest = $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($canonical))
    } finally {
        $sha.Dispose()
    }
    $hex = -join ($digest | ForEach-Object { $_.ToString('x2') })
    return 'sea-forge-' + $hex.Substring(0, 32)
}

# A pipe counts as present when it is listed or accepts a connection. Listing
# alone is not reliable for every .NET runtime.
function Test-PipeExists([string] $Name) {
    if ([System.IO.Directory]::GetFiles('\\.\pipe\') -contains $Name) { return $true }
    $probe = New-Object System.IO.Pipes.NamedPipeClientStream(
        '.', $Name, [System.IO.Pipes.PipeDirection]::InOut,
        [System.IO.Pipes.PipeOptions]::None,
        [System.Security.Principal.TokenImpersonationLevel]::Impersonation)
    try {
        $probe.Connect(300)
        return $true
    } catch {
        return $false
    } finally {
        $probe.Dispose()
    }
}

function Get-SeaForgePipes {
    return ([System.IO.Directory]::GetFiles('\\.\pipe\') | Where-Object { $_ -like 'sea-forge*' }) -join ', '
}

function Wait-Until([scriptblock] $Condition, [int] $Seconds) {
    $deadline = (Get-Date).AddSeconds($Seconds)
    while ((Get-Date) -lt $deadline) {
        if (& $Condition) { return $true }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

# One request line over the pipe, as the current process's user.
function Invoke-PipeRequest([string] $Name, [string] $Request) {
    $client = New-Object System.IO.Pipes.NamedPipeClientStream(
        '.', $Name, [System.IO.Pipes.PipeDirection]::InOut,
        [System.IO.Pipes.PipeOptions]::None,
        [System.Security.Principal.TokenImpersonationLevel]::Impersonation)
    try {
        $client.Connect(10000)
        $writer = New-Object System.IO.StreamWriter($client)
        $writer.AutoFlush = $true
        $writer.Write($Request + "`n")
        $reader = New-Object System.IO.StreamReader($client)
        return $reader.ReadLine()
    } finally {
        $client.Dispose()
    }
}

# A protected verb with an actor block. The case and approval do not exist, so
# an accepted identity yields a non-identity error, never a success.
$ProtectedRequest = '{"verb":"reject","case_id":"e2e-case","approval_id":"e2e-approval","actor":{"actor_id":"operator_local","role":"operator"}}'

# The probe runs as sfprobe. It reports DENIED when the pipe cannot be opened,
# or RESULT with the server's answer.
$ProbeScript = @'
param([string] $PipeName)
$ErrorActionPreference = 'Stop'
$request = '{"verb":"reject","case_id":"e2e-case","approval_id":"e2e-approval","actor":{"actor_id":"operator_local","role":"operator"}}'
try {
    $client = New-Object System.IO.Pipes.NamedPipeClientStream('.', $PipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::None, [System.Security.Principal.TokenImpersonationLevel]::Impersonation)
    $client.Connect(5000)
    $writer = New-Object System.IO.StreamWriter($client)
    $writer.AutoFlush = $true
    $writer.Write($request + "`n")
    $reader = New-Object System.IO.StreamReader($client)
    Write-Output ('RESULT ' + $reader.ReadLine())
} catch {
    Write-Output ('DENIED ' + $_.Exception.GetType().FullName + ': ' + $_.Exception.Message)
}
'@
$probePath = Join-Path $Work 'probe.ps1'
Set-Content -Path $probePath -Value $ProbeScript -Encoding utf8

$runnerSid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
Say "runner SID: $runnerSid"
Say "server: $ServerExe"

$probeUser = 'sfprobe'
$servers = @()
$squatter = $null
$created = $false

function New-Cell([string] $Name) {
    $root = Join-Path $Work $Name
    New-Item -ItemType Directory -Force -Path $root | Out-Null
    $socket = Join-Path $root 'server.sock'
    $yaml = @"
socket_path: '$socket'
identity:
  bindings:
    - sid: '$runnerSid'
      actor_id: operator_local
      roles: [operator]
"@
    Set-Content -Path (Join-Path $root 'server.yaml') -Value $yaml -Encoding utf8
    return [pscustomobject] @{ Root = $root; Socket = $socket; Pipe = (Get-PipeName $socket) }
}

function Start-Server([pscustomobject] $Cell, [string] $Tag) {
    $env:SEA_FORGE_ROOT = $Cell.Root
    Remove-Item Env:\SEA_FORGE_SOCKET -ErrorAction SilentlyContinue
    try {
        return Start-Process -FilePath $ServerExe -PassThru -WindowStyle Hidden `
            -RedirectStandardOutput (Join-Path $Work "$Tag.out.log") `
            -RedirectStandardError (Join-Path $Work "$Tag.err.log")
    } finally {
        Remove-Item Env:\SEA_FORGE_ROOT -ErrorAction SilentlyContinue
    }
}

try {
    # (a) Create the probe user with a random password. The password is never
    # written to the log.
    $alphabet = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789'
    $bytes = [System.Security.Cryptography.RandomNumberGenerator]::GetBytes(24)
    $password = -join ($bytes | ForEach-Object { $alphabet[$_ % $alphabet.Length] }) + 'Aa1!'
    $secure = ConvertTo-SecureString $password -AsPlainText -Force
    $password = $null
    if (Get-LocalUser -Name $probeUser -ErrorAction SilentlyContinue) {
        Remove-LocalUser -Name $probeUser
    }
    New-LocalUser -Name $probeUser -Password $secure -PasswordNeverExpires `
        -UserMayNotChangePassword -AccountNeverExpires | Out-Null
    $created = $true
    Check $true 'probe user created' "($probeUser)"

    # (b) Cell A: the bindings map only the runner's SID to the operator actor.
    $cellA = New-Cell 'cell-a'
    $serverA = Start-Server $cellA 'server-a'
    $servers += $serverA
    $up = Wait-Until { Test-PipeExists $cellA.Pipe } 60
    Check $up 'server A published its pipe' "($($cellA.Pipe))"
    if (-not $up) { Say "pipes listed: [$(Get-SeaForgePipes)]" }
    if (-not $up) { throw 'server A did not start; see server-a logs' }

    # (c) The bound runner user is not refused for identity.
    $raw = Invoke-PipeRequest $cellA.Pipe $ProtectedRequest
    $resp = $raw | ConvertFrom-Json
    $cls = if ($resp.PSObject.Properties.Name -contains 'error_class') { [string] $resp.error_class } else { '' }
    Check (-not $cls.StartsWith('identity_')) 'runner passes identity for a protected verb' "(error_class='$cls')"

    # (d) sfprobe, a different local user, is refused.
    $probeOut = Join-Path $Work 'probe.out.log'
    $probeErr = Join-Path $Work 'probe.err.log'
    $cred = New-Object System.Management.Automation.PSCredential(".\$probeUser", $secure)
    $probe = Start-Process -FilePath 'powershell.exe' -Credential $cred -LoadUserProfile `
        -WorkingDirectory $Work -WindowStyle Hidden -Wait -PassThru `
        -ArgumentList @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $probePath, '-PipeName', $cellA.Pipe) `
        -RedirectStandardOutput $probeOut -RedirectStandardError $probeErr
    $probeLine = (Get-Content -Path $probeOut -ErrorAction SilentlyContinue | Select-Object -Last 1)
    if ($probeLine -match '^DENIED') {
        Check $true 'sfprobe is refused by the pipe DACL' "($($probeLine.Substring(0, [Math]::Min(160, $probeLine.Length))))"
    } elseif ($probeLine -match '^RESULT (.*)$') {
        $pr = $Matches[1] | ConvertFrom-Json
        $pc = if ($pr.PSObject.Properties.Name -contains 'error_class') { [string] $pr.error_class } else { '' }
        Check ($pc.StartsWith('identity_')) 'sfprobe is refused with an identity error' "(error_class='$pc')"
    } else {
        Check $false 'sfprobe outcome recognised' "(exit=$($probe.ExitCode), output='$probeLine')"
    }

    # (e) A second server on the same socket refuses to start.
    $second = Start-Server $cellA 'server-a2'
    $servers += $second
    $exited = Wait-Until { $second.HasExited } 30
    if ($exited) {
        Check ($second.ExitCode -ne 0) 'second server on the same socket refuses to start' "(exit=$($second.ExitCode))"
    } else {
        Check $false 'second server on the same socket refuses to start' '(it kept running)'
        Stop-Process -Id $second.Id -Force -ErrorAction SilentlyContinue
    }
    Check (-not $serverA.HasExited) 'server A still running after the second start'
    $still = Invoke-PipeRequest $cellA.Pipe $ProtectedRequest | ConvertFrom-Json
    $sc = if ($still.PSObject.Properties.Name -contains 'error_class') { [string] $still.error_class } else { '' }
    Check (-not $sc.StartsWith('identity_')) 'server A still answers the bound runner' "(error_class='$sc')"

    Stop-Process -Id $serverA.Id -Force -ErrorAction SilentlyContinue
    Wait-Until { -not (Test-PipeExists $cellA.Pipe) } 20 | Out-Null

    # (f) Squatting: another process owns the derived pipe name before the
    # server starts, so the server must refuse to start.
    $cellB = New-Cell 'cell-b'
    # The squatter writes a marker only after its pipe exists, so the wait does
    # not depend on how the runtime lists or connects to pipes.
    $marker = Join-Path $Work 'squatter.ready'
    Remove-Item -LiteralPath $marker -ErrorAction SilentlyContinue
    $squatCmd = "`$p = New-Object System.IO.Pipes.NamedPipeServerStream('$($cellB.Pipe)', 'InOut', 1); Set-Content -LiteralPath '$marker' -Value ready; Start-Sleep -Seconds 180"
    $squatter = Start-Process -FilePath 'powershell.exe' -PassThru -WindowStyle Hidden `
        -ArgumentList @('-NoProfile', '-Command', $squatCmd)
    $squatted = Wait-Until { Test-Path -LiteralPath $marker } 60
    Check $squatted 'squatter holds the derived pipe name' "($($cellB.Pipe))"
    if ($squatted) {
        $victim = Start-Server $cellB 'server-b'
        $servers += $victim
        $refused = Wait-Until { $victim.HasExited } 30
        if ($refused) {
            Check ($victim.ExitCode -ne 0) 'server refuses to start on a squatted pipe name' "(exit=$($victim.ExitCode))"
        } else {
            Check $false 'server refuses to start on a squatted pipe name' '(it kept running)'
            Stop-Process -Id $victim.Id -Force -ErrorAction SilentlyContinue
        }
    }
}
catch {
    $script:Fails++
    Say "FAIL: unexpected error: $($_.Exception.Message)"
}
finally {
    # (g) Clean up: stop every process started here, then remove the probe user.
    foreach ($server in $servers) {
        if ($server -and -not $server.HasExited) {
            Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
        }
    }
    if ($squatter -and -not $squatter.HasExited) {
        Stop-Process -Id $squatter.Id -Force -ErrorAction SilentlyContinue
    }
    if ($created) {
        Remove-LocalUser -Name $probeUser -ErrorAction SilentlyContinue
        $gone = -not (Get-LocalUser -Name $probeUser -ErrorAction SilentlyContinue)
        Check $gone 'probe user removed' "($probeUser)"
    }
    Say "SUMMARY: $script:Passes passed, $script:Fails failed"
}

if ($script:Fails -gt 0) { exit 1 }
exit 0
