import type { Journey } from "../ladder";
import { J0, J1, J2 } from "./foundation";
import { J3, J4 } from "./representations";
import { J5, J6 } from "./reasoning";
import { J7, J8 } from "./governance";
import { J9, RECOVERY } from "./integrated";

// The affordance-dependency ladder, in order. A journey runs only when everything it depends on
// passed in this run; otherwise it is BLOCKED and names the unsettled dependency.
export const journeys: Journey[] = [J0, J1, J2, J3, J4, J5, J6, J7, J8, J9, RECOVERY];
