// Tooth (2b) support: a Browser whose session, once the operator has signed in, is handed the
// operator's casework_session cookie. The agent-browser session NAMES stay distinct, so only a guard
// that looks at what the browsers actually hold (cookie, actor) can notice.
import { Browser } from "../browser";
import { cookieSetArgs, sessionCookieValue } from "./sessions";

export class CookieLeakBrowser extends Browser {
  private leaked = false;
  constructor(
    session: string,
    private readonly donor: () => Browser | undefined,
    private readonly base: string,
  ) {
    super(session, 30000);
  }

  /** The leak lands the first time the guard inspects this session after it signed in itself. */
  override async cookiesGet(): Promise<string> {
    if (!this.leaked) {
      const donor = this.donor();
      const value = donor ? sessionCookieValue(await donor.cookiesGet()) : null;
      if (!value) throw new Error("cookie-leak tooth: the donor session holds no casework_session cookie to hand over");
      const r = this.run(cookieSetArgs(value, this.base));
      if (r.status !== 0) throw new Error(`cookie-leak tooth: cookies set failed: ${r.stderr}`);
      this.leaked = true;
    }
    return super.cookiesGet();
  }
}
