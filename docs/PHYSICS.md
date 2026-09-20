# Physics and scope

## What is being compared

Both cases assume **uniform linear motion**. All reference frames are inertial, and the relative velocity is constant. No orbital motion, rotation, acceleration, or gravity is simulated. “Root frame” means the inertial coordinates in which a case is initialized and its ray–detector intersection is solved; it does not mean a physically privileged frame.

| Quantity | Telescope-stationary root E | Star-stationary root S |
| --- | --- | --- |
| Telescope and water velocity | 0 | +β along x |
| Star velocity | −β along x | 0 |
| Incoming vacuum ray velocity | (−β, −√(1−β²)) | (0, −1) |
| Detector | Stationary plane | Moving, length-contracted plane |
| Intersection | Calculated in E | Calculated independently in S |

These initial conditions describe corresponding illumination and apparatus orientations. Merely exchanging the words “star” and “Earth” while retaining every coordinate direction does not construct the same experiment. A source's velocity alone does not determine the direction of a ray arriving at the telescope; its emission geometry also matters. We specify the incoming plane wave explicitly.

The distant star is outside the local diagram. Its velocity defines the reference-frame interpretation, while the simulated incoming segment starts upstream of the entrance. The star is not a nearby rendered point whose position could be mistaken for a model of stellar distance or parallax.

## Coordinates and transformation

Use natural units throughout: c = 1, and choose the telescope rest length to be 1. Time and distance share a scale; coordinates are x, y, and t without conversion factors. The GUI displays angles in degrees. Internal calculations and the `--tilt` command-line input use radians. The misalignment control reaches ±0.7 radians (approximately ±40.1°). Let γ = 1/√(1−β²). From E to S:

```text
x  = γ(x′ + βt′)
y  = y′
t  = γ(t′ + βx′)
```

The ray enters at the shared event A = (0, 0, 0). In E its incident unit direction is k′ = (−β, −1/γ). The direction toward the apparent star is −k′, so the apparent angle from +y obeys α = asin β. This is the perpendicular-incidence specialization of relativistic aberration. Einstein's 1905 paper gives the Lorentz transformation, velocity composition, and aberration formula. [Einstein, *On the Electrodynamics of Moving Bodies*, §§3, 5, 7](https://www.fourmilab.ch/etexts/einstein/specrel/www/).

Let δ be the telescope misalignment from the apparent-star direction, and define θ = α + δ. The telescope's inward axis and transverse direction, both in its rest frame, are

```text
d = (−sin θ, −cos θ)
q = ( cos θ, −sin θ)
```

The entrance plane is r′·d = 0 and the detector plane is r′·d = 1. A material point is r′ = ℓd + aq, with ℓ its depth and a its transverse coordinate. The tube half-width defaults to 0.12 and can be adjusted from 0.01 to 0.5 in the toolbar. Deliberately misaligned rays can strike a side wall. The values are illustrative geometry, not measurements of Airy's instrument.

## Case 1: telescope-stationary root

The medium is at rest in this root. Apply Snell's law at its plane entrance:

```text
s = (k′·q)/n = sin δ / n
h = √(1−s²)
u′ = (hd + sq)/n
```

This assumes a homogeneous, isotropic, nondispersive medium with phase and group speeds 1/n in its own rest frame. Reflection changes intensity but is omitted from the transmitted-ray geometry.

Starting at A, the ray follows r′ = u′t′. Intersect the stationary detector:

```text
t′hit = 1 / (u′·d) = n/h
r′hit = u′t′hit
offset = r′hit·q = s/h
```

At alignment δ = 0, s = 0. Therefore r′hit = d for every n: the center is hit, and t′hit = n. Filling the tube changes the transit time without changing the alignment needed to hit the center.

## Case 2: star-stationary root

This solver starts with a vertical incident vacuum ray (0, −1) and a medium moving at (+β, 0). It does **not** call the first solver and transform its detection event.

First transform the incident direction into the medium's rest frame to apply the same constitutive law and boundary refraction. This is required by the model: 1/n is isotropic in the medium rest frame, not generally in the root frame. Transform the resulting propagation velocity back:

```text
ux = (u′x + β)/(1 + βu′x)
uy = u′y/[γ(1 + βu′x)]
```

At a single root time t, a telescope material point has coordinates

```text
x(t) = βt + (ℓdx + aqx)/γ
y(t) = ℓdy + aqy
```

The moving detector plane is therefore

```text
γdx(x − βt) + dyy = 1
```

Substitute the root-frame ray x = uxt, y = uyt and solve directly:

```text
thit = 1/[γdx(ux − β) + dyuy]
xhit = ux thit
yhit = uy thit
```

The physical detector offset is read in the detector's rest coordinates:

```text
offset = qx γ(xhit − βthit) + qy yhit
```

The second panel renders these directly solved S events and directly constructed moving telescope boundaries. It is not a camera change applied to the first solver's output.

## First wall contact and the claimed path

The ray calculations above initially intersect the extended detector plane. Let m be that intersection's signed rest-frame transverse offset and w the tube half-width. Because each post-entry segment is straight and begins at the center, its transverse position increases linearly along the segment. If |m| < w, the first contact is the detector. Otherwise, multiply the entire predicted detector event by w/|m| to obtain the first wall contact. Equality counts as contact with the wall at the detector edge.

The minimum rest-frame wall clearance is max(0, w−|m|). Both root solvers independently supply their ray event and rest-frame offset to this calculation. Rendered paths and playback markers stop at this first contact; a ray that hits a wall is not also reported as a detector arrival.

The optional red comparison is deliberately incorrect. It assigns (ux, uy) = (0, −1/n) in the star root, keeping the incoming direction and imposing the medium-rest speed in a root where the medium moves. It uses the same telescope angle, moving detector and wall geometry as the correct paths. It is drawn only in the root in which this incorrect assumption is defined, and labeled Incorrect model.

This isolates the proposed extra-time, extra-tilt argument; it is not a complete alternative optical theory. A red wall collision is calculated, not guaranteed. At low speed or sufficient width, the red path can instead reach the detector off-center. At zero velocity or n = 1, the aligned incorrect and correct predictions coincide. A positive collision result under an explicitly incorrect law is not evidence for that law.

For this error model, the rest-frame angle that would center the incorrect path is atan(nγβ), compared with the correct angle asin β = atan(γβ). The spurious extra tilt is approximately (n−1)β at small speed. The separate Explanation page develops the argument in accessible language, with these assumptions stated explicitly.

## Independent comparison

Only **after** both intersections have been calculated do the tests and live cross-check Lorentz-transform the E detection event for comparison with the S detection event. The tests cover positive and negative velocities, zero velocity, representative Earth speed, several indices, and deliberate misalignment.

An additional analytic result is available when δ = 0:

```text
E hit = (−β, −1/γ, n)
S hit = (γβ(n−1), −1/γ, γ(n−β²))
```

The detector center passes through each respective hit event. For n = 1, the S ray stays vertical. For n > 1 and β ≠ 0, its horizontal velocity changes inside the moving medium. Setting its root-frame velocity to (0, −1/n) would omit the medium's motion and is not the special-relativistic prediction.

The Minkowski interval t²−x²−y² is invariant. The vacuum segment is null. The effective propagation segment in nondispersive water is timelike, with interval n²−1 for the aligned unit-length tube. This describes propagation in a material, not a claim that a free vacuum photon is massive.

## Simultaneity and playback

The slider follows the entrance clock t′. The corresponding entrance event has S time t = γt′. Each panel intersects its own world tube and ray paths with its own constant-time plane through that entrance event. The two bright telescope outlines are not one set of simultaneous events in both frames.

Travel-time readouts are coordinate times between entry and detection in the panel's root. They are not a single clock's proper-time readings at both locations. Detector offsets are always physical offsets in the telescope rest frame. Two differently timed arrival events can hit the same detector position.

The spacetime boundaries are straight material worldlines and planar world sheets. Each panel clips their display at its own root-time limits; those finite top and bottom edges are not intended to be the Lorentz transforms of the other panel's cutoff events. The underlying surfaces are the same transformed physical boundaries. Regression checks compare independently boosted worldlines with simultaneous spatial polygons, preserve edge intervals and face planarity, and verify that first contacts lie within their corresponding detector or wall faces.

The renderer uses Bevy coordinates (X, Y, Z) = (x, t, y), with equal world-unit scales. Perspective projection and camera rotation change the picture, not the metric or the numerical results. Full paths remain drawn while pulse markers move; endpoints are retained historical events, not pulses that remain in place forever.

Space renders an orthographic x–y snapshot of those same root-time planes, centered on the current tube midpoint. Both panels use the same pixels per unit on x and y, so contraction along x remains visible. Tube-relative traces transport each visited event (ux s, uy s, s) to (ux s + β(t−s), uy s, t) in S; in E the spatial positions stay fixed. The history parameter s runs from entry to min(t, thit). This records passage through the apparatus, not the root-coordinate trajectory flattened onto the current tube. Trace slopes represent tube-relative displacement, not root-frame light velocity. Completed traces move with the tube and terminate at the first contact. Before contact, markers show the actual ray positions; afterward, a marker follows the struck material point (in S: x = xhit + β(t−thit), y = yhit). These retained marks record contacts, not surviving photons. X marks a wall contact; a dot marks a detector contact.

## What Airy measured, and what this model establishes

Airy's 1871 report compared observations at seasons with opposite aberration corrections and rejected the predicted increase caused by the refracting material. It did not report the absence of ordinary stellar aberration. His instrument used a water column and a specially designed objective, so this model's simple tube is not a reconstruction of that optical assembly. [Airy, *On a Supposed Alteration in the Amount of Astronomical Aberration of Light*, pp. 35–39, original paper scan](https://mctoon.net/wp-content/uploads/2019/08/george-airy-1871.pdf), [original publication DOI](https://doi.org/10.1098/rspl.1871.0011).

Here the experiment is simplified to a central ray, plane entrance, and comoving detector. There is no objective lens or focal-plane image formation, window glass, dispersion, diffraction, absorption, atmospheric model, or acceleration. Vacuum is the approximate air reference. Deliberate off-axis offsets demonstrate refraction in this idealization, not a prediction of Airy's micrometer readings.

The demonstrated conclusion is conditional and precise: **special relativity predicts the same physical detector outcome for these corresponding uniformly moving configurations, with no extra pointing angle required by the water.** Airy's null change is therefore not evidence selecting absolute terrestrial rest. The simulation is a derivation and visualization of that prediction, not an independent empirical proof of special relativity or a general model of Earth's shape.

## Aberration readouts

Both panels report the sky angle measured by the telescope observer, relative to +y in its rest frame. The telescope-root solver uses α = asin(β). Independently, the star-root solver starts with its incident velocity (0, −1), applies inverse velocity addition to obtain (−β, −1/γ), and evaluates α = atan2(β, 1/γ). The two values agree, including the sign, without sharing a computed angle. Neither value is an angle measured in the perspective projection of the spacetime diagram. Changing the refractive index or intentional tube misalignment does not change the incident stellar aberration.
