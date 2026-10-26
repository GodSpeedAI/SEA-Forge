/*
 * launcher <argv...>
 *
 * Execs argv[1..] with the inherited environment, waits, and returns the
 * child's exit status (128+signal if the child was killed by a signal).
 * Built with an embedded Info.plist so that the App Sandbox has a bundle id.
 */
#include <errno.h>
#include <spawn.h>
#include <stdio.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

extern char **environ;

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "usage: launcher <argv...>\n");
        return 2;
    }

    pid_t pid;
    int rc = posix_spawnp(&pid, argv[1], NULL, NULL, &argv[1], environ);
    if (rc != 0) {
        fprintf(stderr, "launcher: spawn %s failed: %s\n", argv[1], strerror(rc));
        return 127;
    }

    int status = 0;
    while (waitpid(pid, &status, 0) < 0) {
        if (errno != EINTR) {
            perror("launcher: waitpid");
            return 1;
        }
    }

    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    if (WIFSIGNALED(status)) {
        return 128 + WTERMSIG(status);
    }
    return 1;
}
