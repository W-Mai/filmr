use std::process::Command;
use std::time::{Duration, Instant};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");

    if cmd == "bump" {
        return cmd_bump(args.get(1).map(|s| s.as_str()).unwrap_or(""));
    }
    if cmd == "publish" {
        return cmd_publish(args.iter().any(|a| a == "--dry-run"));
    }
    if cmd == "release" {
        return cmd_release();
    }
    if cmd == "analyze" {
        let path = args.get(1).map(|s| s.as_str()).unwrap_or("");
        return cmd_analyze(path);
    }

    match cmd {
        "ci" => cmd_ci(),
        "build" => cmd_build(),
        "test" => cmd_test(),
        "lint" => cmd_lint(),
        _ => {
            eprintln!(
                "usage: cargo xtask <ci|build|test|lint|bump|publish|release|analyze <image>>"
            );
            std::process::exit(1);
        }
    }
}

// ── CI ───────────────────────────────────────────────────────────────

fn cmd_ci() -> Result {
    for (name, step) in [
        ("build", cmd_build as fn() -> Result),
        ("test", cmd_test),
        ("lint", cmd_lint),
    ] {
        println!("\n=== xtask: {name} ===");
        step()?;
    }
    println!("\n✅ All CI checks passed.");
    Ok(())
}

fn cmd_build() -> Result {
    cargo(&["build", "--workspace", "--all-features", "--all-targets"])
}

fn cmd_test() -> Result {
    cargo(&["test", "--workspace", "--all-features"])
}

fn cmd_lint() -> Result {
    cargo(&["fmt", "--all", "--check"])?;
    cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])
}

// ── Bump ─────────────────────────────────────────────────────────────

fn cmd_bump(level: &str) -> Result {
    if !matches!(level, "major" | "minor" | "patch") {
        return Err("usage: cargo xtask bump <major|minor|patch>".into());
    }
    let root = project_root();
    let current = read_workspace_version(&root)?;
    let next = bump_version(&current, level)?;
    println!("  → bumping {current} → {next}");
    for entry in find_cargo_tomls(&root) {
        if rewrite_version(&entry, &next)? {
            println!("  → updated {entry}");
        }
    }
    println!("  ✅ version bumped to {next}");
    println!("  → run: git add -A && git commit -m \"🔖(release): bump version to {next}\"");
    Ok(())
}

// ── Publish ──────────────────────────────────────────────────────────

fn cmd_publish(dry_run: bool) -> Result {
    let root = project_root();
    ensure_clean_tree(&root)?;

    let mut args = vec!["publish", "-p", "filmr", "--no-verify"];
    if dry_run {
        args.push("--dry-run");
    }
    let verb = if dry_run { "Packaging" } else { "Publishing" };
    println!("  → {verb} filmr");
    cargo(&args)?;

    if dry_run {
        println!("  ✅ dry-run complete");
    } else {
        println!("  ✅ filmr published to crates.io");
    }
    Ok(())
}

// ── Release ──────────────────────────────────────────────────────────

fn cmd_release() -> Result {
    let root = project_root();
    ensure_clean_tree(&root)?;

    let version = read_workspace_version(&root)?;
    let tag = format!("v{version}");
    println!("  → releasing {tag}");

    println!("\n  → git push origin main");
    run_cmd("git", &["push", "origin", "main"])?;

    println!("\n  → waiting for CI (timeout 20min)...");
    wait_for_workflow("rust.yml", Duration::from_secs(20 * 60))?;
    println!("  ✅ CI passed");

    println!("\n  → tagging {tag}");
    tag_and_push(&root, &tag)?;

    println!("\n  → waiting for release workflow (timeout 30min)...");
    wait_for_workflow("release.yml", Duration::from_secs(30 * 60))?;
    println!("  ✅ GitHub Release created");

    println!("\n  → publishing to crates.io...");
    cmd_publish(false)?;

    println!("\n  🎉 released {tag} successfully!");
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

fn project_root() -> String {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(|d| {
            std::path::Path::new(&d)
                .parent()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|_| ".".to_string())
}

fn cargo(args: &[&str]) -> Result {
    run_cmd("cargo", args)
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result {
    println!("  → {cmd} {}", args.join(" "));
    let status = Command::new(cmd)
        .args(args)
        .current_dir(project_root())
        .status()
        .map_err(|e| format!("failed to run {cmd}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd} {} failed with {status}", args.join(" ")).into())
    }
}

fn gh(args: &[&str]) -> Result<String> {
    let out = Command::new("gh")
        .args(args)
        .output()
        .map_err(|e| format!("gh {}: {e}", args.join(" ")))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(format!(
            "gh {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )
        .into())
    }
}

fn ensure_clean_tree(root: &str) -> Result {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()?;
    if !out.stdout.is_empty() {
        return Err("working tree is not clean — commit all changes first".into());
    }
    Ok(())
}

fn wait_for_workflow(workflow: &str, timeout: Duration) -> Result {
    let start = Instant::now();
    println!("  → waiting for {workflow} ...");
    loop {
        std::thread::sleep(Duration::from_secs(15));
        let out = gh(&[
            "run",
            "list",
            "--workflow",
            workflow,
            "--limit",
            "1",
            "--json",
            "status,conclusion",
            "-q",
            ".[0] | [.status, .conclusion] | @tsv",
        ])?;
        let parts: Vec<&str> = out.split('\t').collect();
        let status = parts.first().copied().unwrap_or("");
        let conclusion = parts.get(1).copied().unwrap_or("");
        println!("    {workflow}: {status} / {conclusion}");
        if status == "completed" {
            if conclusion == "success" {
                return Ok(());
            }
            return Err(format!("{workflow} completed with: {conclusion}").into());
        }
        if start.elapsed() > timeout {
            return Err(format!("timeout waiting for {workflow}").into());
        }
    }
}

fn tag_and_push(root: &str, tag: &str) -> Result {
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let tag_commit = Command::new("git")
        .args(["rev-list", "-n1", tag])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    if !tag_commit.is_empty() && tag_commit == head {
        println!("  → tag {tag} already points to HEAD, skipping");
        return Ok(());
    }
    if !tag_commit.is_empty() {
        println!("  → tag {tag} points to old commit, re-tagging...");
        let _ = run_cmd("git", &["tag", "-d", tag]);
        let _ = run_cmd("git", &["push", "origin", &format!(":refs/tags/{tag}")]);
    }
    run_cmd("git", &["tag", tag])?;
    run_cmd("git", &["push", "origin", tag])
}

// ── Version helpers ──────────────────────────────────────────────────

fn read_workspace_version(root: &str) -> Result<String> {
    let content = std::fs::read_to_string(format!("{root}/Cargo.toml"))?;
    content
        .lines()
        .find(|l| l.trim().starts_with("version =") && !l.contains("workspace"))
        .and_then(|l| l.split('"').nth(1))
        .map(|s| s.to_string())
        .ok_or_else(|| "could not find version in workspace Cargo.toml".into())
}

fn bump_version(version: &str, level: &str) -> Result<String> {
    let parts: Vec<u64> = version
        .split('.')
        .map(|p| {
            p.parse::<u64>()
                .map_err(|e| format!("invalid version: {e}"))
        })
        .collect::<std::result::Result<_, _>>()?;
    if parts.len() != 3 {
        return Err(format!("expected semver x.y.z, got {version}").into());
    }
    let (major, minor, patch) = (parts[0], parts[1], parts[2]);
    Ok(match level {
        "major" => format!("{}.0.0", major + 1),
        "minor" => format!("{major}.{}.0", minor + 1),
        "patch" => format!("{major}.{minor}.{}", patch + 1),
        _ => unreachable!(),
    })
}

fn find_cargo_tomls(root: &str) -> Vec<String> {
    let mut result = Vec::new();
    fn walk(dir: &std::path::Path, result: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != "target" && name != "node_modules" && name != ".git" {
                    walk(&path, result);
                }
            } else if path.file_name().is_some_and(|f| f == "Cargo.toml") {
                result.push(path.to_string_lossy().into_owned());
            }
        }
    }
    walk(std::path::Path::new(root), &mut result);
    result.sort();
    result
}

fn rewrite_version(path: &str, next: &str) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;
    let updated = content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let should_rewrite = (trimmed.starts_with("version =")
                && !trimmed.contains("workspace"))
                || (trimmed.contains("path =") && trimmed.contains("version ="));
            if should_rewrite {
                replace_first_semver(line, next)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    if updated == content {
        return Ok(false);
    }
    std::fs::write(path, updated)?;
    Ok(true)
}

fn replace_first_semver(line: &str, next: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find('"') {
        result.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find('"') {
            let inner = &after[..end];
            if is_semver(inner) {
                result.push_str(&format!("\"{next}\""));
                rest = &after[end + 1..];
                result.push_str(rest);
                return result;
            }
            result.push('"');
            result.push_str(inner);
            result.push('"');
            rest = &after[end + 1..];
        } else {
            result.push_str(&rest[start..]);
            return result;
        }
    }
    result.push_str(rest);
    result
}

fn is_semver(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

// ── Analyze ──────────────────────────────────────────────

fn cmd_analyze(path: &str) -> Result {
    if path.is_empty() {
        return Err("usage: cargo xtask analyze <image_path>".into());
    }

    let img = image::open(path)
        .map_err(|e| format!("cannot open {path}: {e}"))?
        .to_rgb8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    let n = w * h;

    let pixels: Vec<[f32; 3]> = img
        .pixels()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();

    let luma: Vec<f32> = pixels
        .iter()
        .map(|p| 0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2])
        .collect();

    let sat: Vec<f32> = pixels
        .iter()
        .map(|p| {
            let mx = p[0].max(p[1]).max(p[2]);
            let mn = p[0].min(p[1]).min(p[2]);
            mx - mn
        })
        .collect();

    println!("Film Stock Analyzer");
    println!("Image: {path}");
    println!("Size: {w}×{h}\n");

    // ── Tonality ──
    println!("═══ Tonality ═══");
    let mut sorted_luma = luma.clone();
    sorted_luma.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |p: usize| sorted_luma[n * p / 100];
    println!(
        "  p1={:.0} p5={:.0} p25={:.0} p50={:.0} p75={:.0} p95={:.0} p99={:.0}",
        pct(1),
        pct(5),
        pct(25),
        pct(50),
        pct(75),
        pct(95),
        pct(99)
    );
    println!("  Dynamic range (p1-p99): {:.0}", pct(99) - pct(1));
    println!("  Contrast (p75-p25): {:.0}", pct(75) - pct(25));

    // ── Color Cast ──
    println!("\n═══ Color Cast (neutral pixels, sat<30) ═══");
    let zones: &[(&str, f32, f32)] = &[
        ("Shadows", 0.0, 60.0),
        ("Midtones", 60.0, 120.0),
        ("Highlights", 120.0, 180.0),
        ("Specular", 180.0, 255.0),
    ];

    println!(
        "  {:12} {:>6} {:>6} {:>6} | {:>5} {:>5} {:>8}",
        "Zone", "R", "G", "B", "R-G", "R-B", "Bias"
    );
    println!("  {}", "-".repeat(55));

    let mut shadow_rg = 0.0f32;
    let mut shadow_rb = 0.0f32;

    for &(name, lo, hi) in zones {
        let mut sr = 0.0f32;
        let mut sg = 0.0f32;
        let mut sb = 0.0f32;
        let mut cnt = 0u32;
        for i in 0..n {
            if sat[i] < 30.0 && luma[i] >= lo && luma[i] < hi {
                sr += pixels[i][0];
                sg += pixels[i][1];
                sb += pixels[i][2];
                cnt += 1;
            }
        }
        if cnt < 50 {
            continue;
        }
        let (r, g, b) = (sr / cnt as f32, sg / cnt as f32, sb / cnt as f32);
        let (rg, rb) = (r - g, r - b);
        if name == "Shadows" {
            shadow_rg = rg;
            shadow_rb = rb;
        }
        let bias = if rg > 2.0 && rb > 2.0 {
            "RED"
        } else if rg < -2.0 && rb < -2.0 {
            "BLUE"
        } else if rg > 2.0 && rb < -2.0 {
            "GREEN"
        } else if rg < -2.0 && rb > 2.0 {
            "MAGENTA"
        } else {
            "neutral"
        };
        println!(
            "  {:12} {:6.1} {:6.1} {:6.1} | {:+5.1} {:+5.1} {:>8}",
            name, r, g, b, rg, rb, bias
        );
    }

    // ── Saturation ──
    println!("\n═══ Saturation ═══");
    let sat_mean: f32 = sat.iter().sum::<f32>() / n as f32;
    let mut sorted_sat = sat.clone();
    sorted_sat.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "  mean={:.1} p50={:.0} p90={:.0} p99={:.0}",
        sat_mean,
        sorted_sat[n / 2],
        sorted_sat[n * 90 / 100],
        sorted_sat[n * 99 / 100]
    );

    // ── Grain ──
    println!("\n═══ Grain (high-pass σ, 7px kernel) ═══");

    // Simple box-filter high-pass per row for grain estimation
    let grain_std = |mask: &[bool]| -> (f32, f32, f32) {
        // Compute noise as pixel - local_mean using a simple sliding window
        let mut sum_r2 = 0.0f64;
        let mut sum_g2 = 0.0f64;
        let mut sum_b2 = 0.0f64;
        let mut cnt = 0u64;
        let radius = 3i32;

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if !mask[idx] {
                    continue;
                }
                let mut lr = 0.0f32;
                let mut lg = 0.0f32;
                let mut lb = 0.0f32;
                let mut lc = 0u32;
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            let ni = ny as usize * w + nx as usize;
                            lr += pixels[ni][0];
                            lg += pixels[ni][1];
                            lb += pixels[ni][2];
                            lc += 1;
                        }
                    }
                }
                let nr = pixels[idx][0] - lr / lc as f32;
                let ng = pixels[idx][1] - lg / lc as f32;
                let nb = pixels[idx][2] - lb / lc as f32;
                sum_r2 += (nr * nr) as f64;
                sum_g2 += (ng * ng) as f64;
                sum_b2 += (nb * nb) as f64;
                cnt += 1;
            }
        }
        if cnt == 0 {
            return (0.0, 0.0, 0.0);
        }
        (
            (sum_r2 / cnt as f64).sqrt() as f32,
            (sum_g2 / cnt as f64).sqrt() as f32,
            (sum_b2 / cnt as f64).sqrt() as f32,
        )
    };

    println!(
        "  {:12} {:>8} {:>6} {:>6} {:>6}",
        "Zone", "Grain σ", "R σ", "G σ", "B σ"
    );
    println!("  {}", "-".repeat(45));

    let mut grain_zones: Vec<(&str, f32)> = Vec::new();
    for &(name, lo, hi) in zones {
        let mask: Vec<bool> = (0..n).map(|i| luma[i] >= lo && luma[i] < hi).collect();
        let count = mask.iter().filter(|&&m| m).count();
        if count < 500 {
            continue;
        }
        let (sr, sg, sb) = grain_std(&mask);
        let avg = (sr + sg + sb) / 3.0;
        grain_zones.push((name, avg));
        println!("  {:12} {:8.2} {:6.2} {:6.2} {:6.2}", name, avg, sr, sg, sb);
    }

    // ── Channel correlation ──
    println!("\n═══ Channel Correlation (midtones) ═══");
    let mid_pixels: Vec<[f32; 3]> = (0..n)
        .filter(|&i| luma[i] >= 80.0 && luma[i] < 180.0)
        .take(10000)
        .map(|i| pixels[i])
        .collect();

    if mid_pixels.len() > 100 {
        let corr = |a: &[f32], b: &[f32]| -> f32 {
            let n = a.len() as f32;
            let ma: f32 = a.iter().sum::<f32>() / n;
            let mb: f32 = b.iter().sum::<f32>() / n;
            let mut cov = 0.0f32;
            let mut va = 0.0f32;
            let mut vb = 0.0f32;
            for i in 0..a.len() {
                let da = a[i] - ma;
                let db = b[i] - mb;
                cov += da * db;
                va += da * da;
                vb += db * db;
            }
            cov / (va.sqrt() * vb.sqrt()).max(1e-10)
        };

        let mr: Vec<f32> = mid_pixels.iter().map(|p| p[0]).collect();
        let mg: Vec<f32> = mid_pixels.iter().map(|p| p[1]).collect();
        let mb: Vec<f32> = mid_pixels.iter().map(|p| p[2]).collect();
        let rg = corr(&mr, &mg);
        let rb = corr(&mr, &mb);
        let gb = corr(&mg, &mb);
        let avg_corr = (rg + rb + gb) / 3.0;
        println!(
            "  R-G={:.3}  R-B={:.3}  G-B={:.3}  avg={:.3}",
            rg, rb, gb, avg_corr
        );

        // ── PSD shape analysis (Schallauer ICIP 2006) ──
        println!("\n═══ PSD Shape (radial power spectrum) ═══");
        use rayon::prelude::*;
        use rustfft::{num_complex::Complex, FftPlanner};

        let blk = 64usize;
        let half = blk / 2;

        // Collect candidate block positions
        let grid_step = blk; // non-overlapping grid
        let block_positions: Vec<(usize, usize)> = (0..(h / grid_step))
            .flat_map(|gy| (0..(w / grid_step)).map(move |gx| (gx * grid_step, gy * grid_step)))
            .filter(|&(bx, by)| {
                if bx + blk > w || by + blk > h {
                    return false;
                }
                let bc = (blk * blk) as f64;
                let (mut s, mut s2) = (0.0f64, 0.0f64);
                for dy in 0..blk {
                    for dx in 0..blk {
                        let v = luma[(by + dy) * w + (bx + dx)] as f64;
                        s += v;
                        s2 += v * v;
                    }
                }
                (s2 / bc - (s / bc).powi(2)).sqrt() < 20.0
            })
            .collect();

        // Parallel PSD + isotropy per block
        let block_results: Vec<(Vec<f64>, f32)> = block_positions
            .par_iter()
            .map(|&(bx, by)| {
                let mut planner = FftPlanner::<f64>::new();
                let fft = planner.plan_fft_forward(blk);
                let mut rows: Vec<Vec<Complex<f64>>> = (0..blk)
                    .map(|dy| {
                        (0..blk)
                            .map(|dx| Complex::new(luma[(by + dy) * w + (bx + dx)] as f64, 0.0))
                            .collect()
                    })
                    .collect();
                for row in rows.iter_mut() {
                    fft.process(row);
                }
                let mut cols: Vec<Vec<Complex<f64>>> = (0..blk)
                    .map(|c| (0..blk).map(|r| rows[r][c]).collect())
                    .collect();
                for col in cols.iter_mut() {
                    fft.process(col);
                }
                // Radial PSD
                let mut radial = vec![0.0f64; half];
                let mut rcnt = vec![0u32; half];
                let mut dir_power = [0.0f64; 4];
                let mut dir_cnt = [0u32; 4];
                for (fy, col) in cols.iter().enumerate() {
                    for (fx, coeff) in col.iter().enumerate() {
                        let cy = if fy < half { fy } else { blk - fy };
                        let cx = if fx < half { fx } else { blk - fx };
                        let r = ((cx * cx + cy * cy) as f64).sqrt() as usize;
                        let power = coeff.norm_sqr();
                        if r > 0 && r < half {
                            radial[r] += power;
                            rcnt[r] += 1;
                        }
                        // Isotropy
                        let cyi = if fy < half {
                            fy as i32
                        } else {
                            fy as i32 - blk as i32
                        };
                        let cxi = if fx < half {
                            fx as i32
                        } else {
                            fx as i32 - blk as i32
                        };
                        let r2 = cxi * cxi + cyi * cyi;
                        if r2 >= 4 && r2 <= (half as i32 * half as i32) {
                            let angle = (cyi as f64)
                                .atan2(cxi as f64)
                                .to_degrees()
                                .rem_euclid(180.0);
                            let wedge = ((angle + 22.5) / 45.0) as usize % 4;
                            dir_power[wedge] += power;
                            dir_cnt[wedge] += 1;
                        }
                    }
                }
                for i in 0..half {
                    if rcnt[i] > 0 {
                        radial[i] /= rcnt[i] as f64;
                    }
                }
                let dp: Vec<f64> = (0..4)
                    .map(|i| {
                        if dir_cnt[i] > 0 {
                            dir_power[i] / dir_cnt[i] as f64
                        } else {
                            0.0
                        }
                    })
                    .collect();
                let dp_mean = dp.iter().sum::<f64>() / 4.0;
                let dir = if dp_mean > 0.0 {
                    let mx = dp.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let mn = dp.iter().cloned().fold(f64::INFINITY, f64::min);
                    ((mx - mn) / dp_mean) as f32
                } else {
                    0.0
                };
                (radial, dir)
            })
            .collect();

        // Aggregate
        let psd_cnt = block_results.len();
        let mut psd_accum = vec![0.0f64; half];
        let mut dir_scores: Vec<f32> = Vec::with_capacity(psd_cnt);
        for (radial, dir) in &block_results {
            for i in 0..half {
                psd_accum[i] += radial[i];
            }
            dir_scores.push(*dir);
        }

        let psd_rolloff = if psd_cnt > 0 {
            let lo = psd_accum[1..half / 4].iter().sum::<f64>() / (half / 4 - 1).max(1) as f64;
            let hi = psd_accum[half * 3 / 4..half].iter().sum::<f64>() / (half / 4).max(1) as f64;
            if lo > 0.0 {
                (hi / lo) as f32
            } else {
                1.0
            }
        } else {
            1.0
        };
        println!(
            "  PSD high/low ratio: {:.3} (film<0.3, digital≈1.0)",
            psd_rolloff
        );

        let isotropy = if !dir_scores.is_empty() {
            dir_scores.iter().sum::<f32>() / dir_scores.len() as f32
        } else {
            0.0
        };
        println!(
            "  Directionality: {:.3} (film<0.3 isotropic, texture>0.5)",
            isotropy
        );

        // ── Signal dependency curve (sampled) ──
        println!("\n═══ Signal Dependency (grain σ vs intensity) ═══");
        let num_bins = 8usize;
        let sd_step = ((w * h) / 50000).max(1);
        let chunk_size = ((w * h) / sd_step / rayon::current_num_threads()).max(1);
        let indices: Vec<usize> = (0..(w * h)).step_by(sd_step).collect();
        let partial: Vec<(Vec<f64>, Vec<u64>)> = indices
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut bs = vec![0.0f64; num_bins];
                let mut bc = vec![0u64; num_bins];
                let radius = 3i32;
                for &idx in chunk {
                    let x = idx % w;
                    let y = idx / w;
                    let l = luma[idx];
                    let bin = ((l / 256.0) * num_bins as f32).min(num_bins as f32 - 1.0) as usize;
                    let mut lr = 0.0f32;
                    let mut lc = 0u32;
                    for dy in -radius..=radius {
                        for dx in -radius..=radius {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                                lr += luma[ny as usize * w + nx as usize];
                                lc += 1;
                            }
                        }
                    }
                    let noise = luma[idx] - lr / lc as f32;
                    bs[bin] += (noise * noise) as f64;
                    bc[bin] += 1;
                }
                (bs, bc)
            })
            .collect();
        let mut bin_sigma = vec![0.0f64; num_bins];
        let mut bin_count = vec![0u64; num_bins];
        for (bs, bc) in &partial {
            for i in 0..num_bins {
                bin_sigma[i] += bs[i];
                bin_count[i] += bc[i];
            }
        }
        let sigmas: Vec<f32> = (0..num_bins)
            .map(|i| {
                if bin_count[i] > 0 {
                    (bin_sigma[i] / bin_count[i] as f64).sqrt() as f32
                } else {
                    0.0
                }
            })
            .collect();
        for (i, sigma) in sigmas.iter().enumerate() {
            let lo = i * 256 / num_bins;
            let hi = (i + 1) * 256 / num_bins - 1;
            println!("  [{:3}-{:3}]: σ={:.2}", lo, hi, sigma);
        }
        let peak_bin = sigmas
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        let is_bell = (2..=5).contains(&peak_bin)
            && sigmas[0] < sigmas[peak_bin] * 0.8
            && sigmas[num_bins - 1] < sigmas[peak_bin] * 0.8;
        println!(
            "  Peak at bin {}/{} → {}",
            peak_bin,
            num_bins,
            if is_bell {
                "bell-shaped (film characteristic)"
            } else {
                "not bell-shaped"
            }
        );

        // ── Film detection ──
        println!("\n═══ Film Detection ═══");
        let mut score = 0.0f32;
        let mut reasons: Vec<&str> = Vec::new();

        // 1. Grain intensity: film shadows σ > 15
        let shadow_grain = grain_zones.first().map(|z| z.1).unwrap_or(0.0);
        if shadow_grain > 15.0 {
            score += 25.0;
            reasons.push("strong grain in shadows");
        } else if shadow_grain > 8.0 {
            score += 10.0;
            reasons.push("moderate grain");
        }

        // 2. Channel decorrelation: film < 0.85
        if avg_corr < 0.7 {
            score += 25.0;
            reasons.push("low channel correlation (independent RGB grain)");
        } else if avg_corr < 0.85 {
            score += 15.0;
            reasons.push("moderate channel decorrelation");
        }

        // 3. Grain varies with luminance (negative film: shadows > highlights)
        let highlight_grain = grain_zones.last().map(|z| z.1).unwrap_or(0.0);
        if shadow_grain > 1.0 && highlight_grain > 0.1 {
            let ratio = shadow_grain / highlight_grain.max(0.01);
            if ratio > 2.0 {
                score += 25.0;
                reasons.push("grain increases in shadows (negative film characteristic)");
            } else if ratio > 1.3 {
                score += 10.0;
                reasons.push("slight grain-luminance dependence");
            }
        }

        // 4. Compressed dynamic range (film scan typically < 220)
        let dr = pct(99) - pct(1); // p99 - p1
        if dr < 200.0 {
            score += 15.0;
            reasons.push("compressed dynamic range");
        } else if dr < 230.0 {
            score += 5.0;
            reasons.push("slightly compressed DR");
        }

        // 5. Shadow color cast (film base + fog)
        let shadow_cast = shadow_rg.abs().max(shadow_rb.abs());
        if shadow_cast > 8.0 {
            score += 10.0;
            reasons.push("shadow color cast (film base)");
        }

        // 6. PSD rolloff (film grain is band-limited)
        if psd_rolloff < 0.15 {
            score += 15.0;
            reasons.push("strong PSD rolloff (band-limited grain)");
        } else if psd_rolloff < 0.35 {
            score += 8.0;
            reasons.push("moderate PSD rolloff");
        }

        // 7. Isotropy (film grain is isotropic)
        if isotropy < 0.15 {
            score += 10.0;
            reasons.push("isotropic noise (film grain)");
        } else if isotropy < 0.3 {
            score += 5.0;
            reasons.push("mostly isotropic");
        }

        // 8. Bell-shaped signal dependency
        if is_bell {
            score += 15.0;
            reasons.push("bell-shaped grain curve (film characteristic)");
        }

        let verdict = if score >= 60.0 {
            "Very likely FILM scan"
        } else if score >= 35.0 {
            "Possibly film or heavy film simulation"
        } else {
            "Likely DIGITAL"
        };
        println!("  Score: {:.0}/100 → {}", score.min(100.0), verdict);
        if !reasons.is_empty() {
            println!("  Evidence: {}", reasons.join("; "));
        }
    }

    Ok(())
}
