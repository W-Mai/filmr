# Photochemistry Core (Quantum Scale)

When photons strike a silver halide crystal (AgX), a photolytic reaction is triggered:

## 1.1 Photoelectron Excitation

$$ \text{AgX} + h\nu \xrightarrow{k_{\text{photo}}} \text{Ag}^+ + \text{e}^- + \text{X}^0 $$

Where:
- $h\nu$ is the photon energy ($\nu$ is frequency)
- $k_{\text{photo}}$ is the photochemical reaction rate constant, following the quantum efficiency model:
  $$ k_{\text{photo}} = \eta \cdot \sigma_{\text{abs}} \cdot \Phi $$
  - $\eta$: Quantum efficiency (≈0.6-0.9)
  - $\sigma_{\text{abs}}$: Crystal absorption cross-section
  - $\Phi$: Photon flux (photons/(cm²·s))

## 1.2 Latent Image Center Formation

Released electrons are trapped by silver ions, forming silver atom clusters:

$$ \text{Ag}^+ + \text{e}^- \xrightarrow{k_{\text{trap}}} \text{Ag}^0 $$

**Critical Condition**: A **stable latent image center** is formed when a single crystal surface accumulates **5-10 silver atoms**:

$$ n_{\text{Ag}} \geq n_{\text{critical}} \approx 5-10 $$

This process can be modeled using a **Poisson distribution**:

$$ P(n_{\text{Ag}} \geq n_{\text{crit}}) = 1 - \sum_{k=0}^{n_{\text{crit}}-1} \frac{\lambda^k e^{-\lambda}}{k!} $$

Where $\lambda = \eta \cdot N_{\text{photons}}$ is the expected number of electrons.

---

# Exposure and Density Mapping (Macro Scale)

## 2.1 Exposure Definition

$$ E = I \cdot t $$

- $I$: Irradiance (lux)
- $t$: Exposure time (s)
- $E$: Exposure (lux·s)

## 2.2 Optical Density

The ability of silver grains to block light after development is quantified by **Optical Density** $D$:

$$ D = \log_{10}\left(\frac{I_0}{I_t}\right) = -\log_{10}(T) $$

- $I_0$: Incident light intensity
- $I_t$: Transmitted light intensity
- $T = I_t/I_0$: Transmittance

---

# Characteristic Curve (Hurter-Driffield Curve)

This is the **core mathematical model** of film imaging, describing the S-shaped relationship between $D$ and $\log_{10}E$:

## 3.1 Piecewise Linear Model (Engineering Simplification)

$$
D(\log E) = \begin{cases}
D_{\text{min}} + \frac{\log E - \log E_{\text{toe}}}{\gamma_{\text{toe}}} & \text{Toe (Underexposed)} \\
D_{\text{min}} + \gamma \cdot (\log E - \log E_0) & \text{Linear Region (Normal Exposure)} \\
D_{\text{max}} - \frac{\log E_{\text{shoulder}} - \log E}{\gamma_{\text{shoulder}}} & \text{Shoulder (Overexposed)}
\end{cases}
$$

**Key Parameters**:
- $\gamma$: Contrast coefficient (slope of the linear region)
  $$ \gamma = \frac{\Delta D}{\Delta \log E} = \frac{D_2 - D_1}{\log E_2 - \log E_1} $$
- Dynamic Range: $\text{DR} = \log_{10}(E_{\text{max}}/E_{\text{min}}) \approx 2.0$ (corresponding to 100:1 brightness ratio)

## 3.2 Precise Error Function Model (Scientific Grade)

Analytical solution provided by Kodak technical documents:

$$
D(E) = \frac{D_0}{2}\left[1 + \text{erf}\left(X + \ln\frac{E - E_0}{E_g - E_0}\right)\right]
$$

Where the error function is:

$$
\text{erf}(y) = \frac{1}{\sqrt{\pi}} \int_{-\infty}^{y} e^{-z^2} dz
$$

Parameter meanings:
- $D_0$: Maximum saturation density
- $E_0$: Threshold exposure
- $E_g$: Sensitivity reference point
- $X$: Development condition correction term

---

# Development Kinetics (Chemical Amplification)

Development amplifies latent image centers by $10^6$-$10^8$ times. Its rate equation:

## 4.1 First-Order Kinetics Model

$$
\frac{d[\text{Ag}]}{dt} = k_{\text{dev}} \cdot [\text{Dev}] \cdot [\text{Ag}^+] \cdot N_{\text{latent}}
$$

- $[\text{Dev}]$: Developer concentration
- $N_{\text{latent}}$: Number of latent image centers (proportional to exposure)

## 4.2 Time-Temperature Compensation (Arrhenius Equation)

$$
k_{\text{dev}} = A \cdot e^{-E_a/(RT)}
$$

- $E_a$: Activation energy (≈50-70 kJ/mol)
- $R$: Gas constant
- $T$: Absolute temperature (K)
