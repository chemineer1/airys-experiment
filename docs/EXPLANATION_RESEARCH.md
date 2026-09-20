# Research behind the explanation

Reviewed 20 September 2026. The public explanation is `EXPLANATION.md`; these notes record the source selection and limits behind the rewrite.

## The claim and its provenance

The claim is that slowing the light in a water-filled telescope should require extra tilt if Earth moves, so unchanged aberration indicates stationary Earth. This is a later interpretation of Airy's observations, not the conclusion of his report.

- [The Flat Earth Wiki, archived page](https://soundquality.org/wp-content/uploads/2024/03/Airys-Failure-The-Flat-Earth-Wiki.pdf) explicitly presents the stars-moving/Earth-stationary inference. The archived page identifies its last modification as October 2023.
- [Walter van der Kamp, Airy's Failure Reconsidered](https://www.geocentricity.com/ba1/no066/vdkamp.html) is an earlier geocentric presentation of the extra-tilt claim. Its HTML conversion date is not its original publication date.
- [The organization's 1998 editorial](https://www.geocentricity.com/ba1/no084/editor.html) dates van der Kamp's *Airy Reconsidered* to composition in 1968 and distribution in 1970. This supports twentieth-century provenance, but does not establish the argument's earliest origin.
- [Plane Geodesy](https://planegeodesy.com/heliocentrism-refuted-the-airy-experment-1871) supplies a further contemporary example of interpreting Airy's rejection of a particular optical prediction as rejection of Earth's orbital motion.

These sources establish what proponents claim. They are not used as authorities on optics or on what the experimental evidence establishes.

## Airy's actual experiment

[Airy's 1871 report](https://mctoon.net/wp-content/uploads/2019/08/george-airy-1871.pdf), Proceedings of the Royal Society of London 20, 35–39, DOI [10.1098/rspl.1871.0011](https://doi.org/10.1098/rspl.1871.0011), is the principal source. Read the original scan, including the results table and conclusion, rather than relying on the modern label “failure.”

It explicitly addresses Klinkerfues's predicted increase in aberration. Airy observed Gamma Draconis with a purpose-built water-filled optical instrument and compared spring and autumn results using tables with ordinary aberration corrections. He rejected the additional effect. The distinction between ordinary aberration and additional aberration must survive every summary.

The [Royal Museums Greenwich collection record](https://www.rmg.co.uk/collections/objects/rmgc-object-11162) identifies the surviving instrument as a special zenith sector, object AST1000. This is a historical optical instrument with lenses, not a literal reconstruction of the simulation's open central-ray geometry.

## Earlier optical history

[Pedersen (2000)](https://pure.au.dk/portal/en/publications/water-filled-telescopes-and-the-pre-history-of-fresnels-ether-dra/) researches the eighteenth- and nineteenth-century proposals. Its abstract establishes that multiple earlier investigators expected unchanged aberration. [Antonello, Water-filled telescopes](https://arxiv.org/pdf/1401.5585) provides a further historical survey and bibliography.

The explainer therefore avoids suggesting that all older optical theories predicted extra tilt or that relativity was invented solely to rescue Airy's result. It does not claim that this experiment uniquely establishes relativity.

## Relativistic calculation

Primary theoretical sources:

- [Einstein (1905), §§3, 5, 7](https://www.fourmilab.ch/etexts/einstein/specrel/www/): coordinate transformations, velocity addition, and aberration.
- [Laue (1907), English translation](https://en.wikisource.org/wiki/Translation:The_Entrainment_of_Light_by_Moving_Bodies_in_Accordance_with_the_Principle_of_Relativity), [original publication](https://doi.org/10.1002/andp.19073281015): light in moving media from relativistic velocity addition, with attention to dispersion.

The explicit transverse tube geometry is this project's derivation, independently reviewed against `src/physics.rs`. It uses normal incidence in the telescope rest frame, uniform linear motion, a homogeneous isotropic nondispersive medium, and a detector moving with the tube. Relativity guarantees agreement about encounters; these optical assumptions establish the centered ray independently of refractive index.

The proof establishes the full centerline identity throughout the flight, not merely matching endpoint coordinates. It uses exact expressions, not a first-order speed approximation. The standard longitudinal Fresnel coefficient must not be substituted for the transverse velocity in this geometry.

The red illustrative model holds the transmitted ray vertical at speed 1/n in star coordinates. It exposes the specific omitted moving-medium effect. Its agreement at zero speed or n = 1 is explicitly retained; a miss need not be a wall collision for a sufficiently wide tube.


## Video-specific argument and source checks

Reviewed Malcolm Bowden's [GEOCENTRICITY animation](https://www.youtube.com/watch?v=87M2i61N1cU) using the complete auto-generated English transcript and visual inspection of the relevant animation and report excerpts. The transcript is a navigation aid; the numerical claims were checked against the displayed report and original scan. No full transcript or video frames are reproduced in this repository.

- 0:22–1:13: introduces the initial tilt, then proposes ether carrying stars and starlight around a stationary Earth. This is more specific than merely changing the star's velocity.
- 1:45–2:19: depicts a water-filled telescope moving sideways under a vertical incoming ray, then additional tilt. The illustrated 5° and 10° values are not Airy's measured aberration.
- 2:24–3:05: contrasts a stationary telescope receiving an angled ray that slows without changing its path; claims no added tilt is needed only here.
- 4:25–4:42: infers a stationary Earth from the unchanged result. The rebuttal identifies the missing transverse transmitted velocity in the moving-water case. The simulation's two frames describe corresponding illumination, not arbitrary changes to source motion or a physical ether theory.
- 3:32–3:46: treats a predicted 30-arcsecond discrepancy as a general consequence of Earth moving. Printed page 38 of Airy's report explicitly attributes this discrepancy to Klinkerfues's hypothesis: added seasonal corrections of opposite signs, each 15 arcseconds. The compared table values are inferred spring/autumn instrument latitudes, not direct empty-versus-water readings.
- 3:51–4:20: interprets the diminishing 100-arcsecond quantity as evidence for declining light speed. Printed page 36 explicitly identifies it as Gamma Draconis's mean zenith distance north at Greenwich. It is neither the aberration amplitude nor a measured light speed.

The report does not support the video's attribution of a stationary-Earth conclusion to Airy. The explanation addresses the optical inference and the specific report misreadings without speculating about the narrator's motives or reproducing his unrelated closing claims.
