# AI tells: critique checklist

Run this on every finished screen. A match is not automatically wrong, but it must be **justified by a content reason** (the data or the task needs it) or removed. "It looks nice" is not a reason.

## Visual tells

| Tell | Why it reads as generated | Ask instead |
|---|---|---|
| Gradients on backgrounds, buttons, or text | Decoration with no meaning | What does this color change communicate? Use flat tokens. |
| Glassmorphism, blur, glows, neon borders | Effect standing in for design | What is floating above what, and why? |
| Soft shadows on every card | Elevation everywhere means hierarchy nowhere | Would spacing and a divider do the grouping? |
| Everything rounded the same large radius | One setting applied to all | Does the radius follow a component class decision? |
| Cards for everything | Boxes replace structure | Is this a row, a section, or truly a self-contained object? |
| Emoji or generic icons as decoration | Filler | Does the icon carry meaning that the label does not? |
| An icon beside every heading | Pattern, not purpose | Remove; keep icons only where they speed recognition. |
| Colored pill badges everywhere | Status noise | Is this a real status that changes what the person does? |
| Stock framework palette untouched (default zinc, indigo, etc.) | No decision was made | Which role tokens did the owner choose, and why? |
| Many font weights and sizes | Hierarchy by accident | Reduce to the scale; use weight and color deliberately. |
| Excess animation and hover effects | Motion as polish | Does the motion explain a change of state? |

## Layout tells

| Tell | Why | Ask instead |
|---|---|---|
| Centered hero with headline, subtitle, two buttons | Marketing template on a product screen | What is the person here to do? Put that first. |
| Row of three feature boxes | Template filler | Is there real content to show instead? |
| Everything centered | Avoids deciding alignment | Left-align text; center only short single-purpose content. |
| Identical padding on everything | No grouping logic | Is space between groups larger than within? |
| Dashboard with stat tiles when no data exists | Fake importance | What does the person actually need to see at this stage? |
| Wide empty margins with a tiny card in the middle | Layout by default | Is the content width chosen for line length? |
| Full-width forms with wide fields | Ignores measure | Match field width to expected input. |

## Content tells

| Tell | Ask instead |
|---|---|
| Copy like "Welcome back!", "Seamless", "Powerful", "Unlock", "Effortlessly" | Say what the screen does in plain words, in the roadmap's vocabulary (person, offer, commitment). |
| Lorem ipsum or invented sample data that is always perfect length | Use real content at short, typical, and very long lengths. |
| Placeholder as label | Visible labels on every field. |
| Generic error text ("Something went wrong") | What happened and what can the person do now? |
| Exclamation marks and cheerfulness on serious actions | Calm, neutral tone, especially for deletion and errors. |
| Invented features on screen (filters, tags, ratings the roadmap has not reached) | Only what the current stage and schema support. |

## Structural tells

- A component with several optional props that switch it into different things: split it.
- Styles that differ per page for the same component: the component is not owning its look.
- Magic values (a one-off 13px, a stray color): add a token with a reason or use an existing one.
- A page that had to change because one component improved: the interface between them is wrong.

## How to report

For each match: where, which tell, the content reason offered, and the verdict (kept with reason, or removed). End with the count of tells removed and any left, so the owner can overrule.
