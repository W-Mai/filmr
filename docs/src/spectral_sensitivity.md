# Spectral Intuition and Color Shift

---

## 1. Key Premise Clarification

> **Film Simulation ≠ Color Preservation Algorithm**

Film simulation typically **allows for overall tonal changes** (such as warm tones, cool tones, curve compression),
**but does NOT allow:**

* Unexpected channel crosstalk (e.g., Red being asymmetrically pulled)
* Neutral colors becoming non-neutral
* Uncontrollable hue shifts related to luminance

So the goal is not "Output = Input", but rather:

> **Verify if there are abnormal color shifts within the "known acceptable range of variation"**

---

## 2. Core Idea: Use "Diagnostic Images"

Natural photos **cannot be used for algorithm verification** because you can never prove whether the color cast comes from the algorithm or the content itself.

You need **Synthetic / Diagnostic Images**.

---

## 3. Category 1: Neutral Axis Verification (Most Important)

### 1️⃣ Input: Ideal Neutral Grayscale

Construct an image:

* R = G = B
* From 0 → 255 (or linear 0 → 1)
* Include different luminance sections (Shadows / Mid-gray / Highlights)

### Expected Behavior

* After film simulation:
  * **R′ ≈ G′ ≈ B′**
  * Overall brightening/darkening/curve changes are allowed
  * **Systematic deviation of Δ(R′−G′) with luminance is NOT allowed**

### Calculable Metrics

#### Metric A: Neutral Color Deviation

```text
Δ_neutral = mean(|R' - G'| + |G' - B'| + |B' - R'|)
```

#### Metric B: Luminance-Correlated Deviation

```text
corr(L_in, (R'-G'))
corr(L_in, (G'-B'))
```

> If color cast changes with luminance → **Typical Algorithmic Error**
> (Commonly seen when incorrect curves are applied in RGB space instead of luminance space)

---

## 4. Category 2: Pure Color Channel Integrity Test

### 2️⃣ Input: Pure Colors and Single Channel Gradients

Construct:

* (R=1, G=0, B=0)
* (0,1,0)
* (0,0,1)
* And gradients for each channel 0→1

### Expected Behavior

* Film allows:
  * Pure Red → Shifts to Orange / Magenta (This is style)
* **But it must be explainable and symmetric**

### Calculable Metrics

#### Metric C: Channel Leakage Matrix

You can approximate a 3×3 matrix:

```text
[ R' ]   [ a b c ] [ R ]
[ G' ] ≈ [ d e f ] [ G ]
[ B' ]   [ g h i ] [ B ]
```
