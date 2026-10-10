import type { Journey } from "../ladder";
import { L0 } from "./L0";
import { L1 } from "./L1";
import { L2 } from "./L2";
import { L3 } from "./L3";
import { L4 } from "./L4";
import { L5 } from "./L5";
import { L6 } from "./L6";
import { L7 } from "./L7";

// The live ladder (plan T10). Journeys settle in order; each asserts its UI AND the durable delta.
export const liveJourneys: Journey[] = [L0, L1, L2, L3, L4, L5, L6, L7];
