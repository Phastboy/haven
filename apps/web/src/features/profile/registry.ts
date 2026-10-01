import type { ProfileSection } from "./types";
import Completeness from "./sections/Completeness.astro";
import Header from "./sections/Header.astro";
import About from "./sections/About.astro";

/**
 * The ONLY place a section is registered. ProfileView never changes.
 * Add a section: create sections/<Name>.astro, import it, add one line.
 */
export const sections: ProfileSection[] = [
  { id: "header", order: 0, audience: "both", component: Header },
  { id: "completeness", order: 5, audience: "owner", component: Completeness },
  { id: "about", order: 10, audience: "both", component: About },
];
