pub mod helper {
    use crate::perf;

    pub fn f() {
        perf!("hi");
    }
}

use proconio::input;

use crate::helper::f;

params_impl! {
    n_coef: f64 = 3.9572835556899086,
    m_coef: f64 = 7.0385499979441235,
}

// fn group_id_fn(n: usize, m: usize) -> String {
//     if n < 50 && m < 50 {
//         "0".to_string()
//     } else if n < 101 && m < 50 {
//         "1".to_string()
//     } else if n < 50 && m < 101 {
//         "2".to_string()
//     } else {
//         "3".to_string()
//     }
// }

// params_impl! {
//     { n_coef: f64, m_coef: f64 },
//     [
//         "0" => { n_coef: 5.911064041309099, m_coef: 3.404873337327698 },
//         "1" => { n_coef: 2.574560011770543, m_coef: 5.910610920190062 },
//         "2" => { n_coef: 3.5737121598862833, m_coef: 7.571256449897406 },
//         "3" => { n_coef: 4.728672779566716, m_coef: 8.0125115098411 },
//         _ => { n_coef: 5.0, m_coef: 5.0 },
//     ]
// }

fn main() {
    input! {
        n: usize,
        m: usize,
    }
    let params = Params::load();
    // let group_id = group_id_fn(n, m);
    // let params = Params::load(&group_id);

    let x = n as i64 * params.n_coef as i64 + m as i64 * params.m_coef as i64;

    println!("{}", x);

    dumpln!("hi");

    f();
}

pub mod ahc_library {
    pub mod annealer {
        pub mod components {
            pub mod criterion {
                use crate::ahc_library::{annealer::types::Criterion, utils::random::Random};

                pub struct HillClimbingCriterion {
                    is_maximize: bool,
                }

                impl HillClimbingCriterion {
                    pub fn new(is_maximize: bool) -> Self {
                        HillClimbingCriterion { is_maximize }
                    }
                }

                impl Criterion for HillClimbingCriterion {
                    fn adopt(
                        &self,
                        cur_score: f64,
                        new_score: f64,
                        _: f64,
                        _: f64,
                        _: &mut impl Random,
                    ) -> bool {
                        if self.is_maximize {
                            new_score >= cur_score
                        } else {
                            new_score <= cur_score
                        }
                    }
                }

                pub struct AnnealingCriterion {
                    is_maximize: bool,
                }

                impl AnnealingCriterion {
                    pub fn new(is_maximize: bool) -> Self {
                        AnnealingCriterion { is_maximize }
                    }
                }

                impl Criterion for AnnealingCriterion {
                    fn adopt(
                        &self,
                        cur_score: f64,
                        new_score: f64,
                        cur_temp: f64,
                        _: f64,
                        rnd: &mut impl Random,
                    ) -> bool {
                        let sign = self.is_maximize as i32 * 2 - 1;
                        let score_diff = sign as f64 * (new_score - cur_score);
                        if score_diff > 0. {
                            return true;
                        }
                        let prob = (score_diff / cur_temp).exp();
                        rnd.nextf() < prob
                    }
                }
            }

            pub mod temperature_scheduler {
                use crate::ahc_library::annealer::types::TemperatureScheduler;

                pub struct ExpTemperatureScheduler {
                    start_temp: f64,
                    end_temp: f64,
                }

                impl ExpTemperatureScheduler {
                    pub fn new(start_temp: f64, end_temp: f64) -> Self {
                        ExpTemperatureScheduler {
                            start_temp,
                            end_temp,
                        }
                    }
                }

                impl TemperatureScheduler for ExpTemperatureScheduler {
                    fn get_temp(&self, progress: f64) -> f64 {
                        self.start_temp.powf(1. - progress) * self.end_temp.powf(progress)
                    }
                }
            }

            pub mod progress_scheduler {
                use crate::ahc_library::{annealer::types::ProgressScheduler, utils::time};

                pub struct IterationProgressScheduler {
                    iteration: usize,
                    cur_step: usize,
                }

                impl IterationProgressScheduler {
                    pub fn new(iteration: usize) -> Self {
                        IterationProgressScheduler {
                            iteration,
                            cur_step: 0,
                        }
                    }
                }

                impl ProgressScheduler for IterationProgressScheduler {
                    fn step(&mut self) {
                        self.cur_step += 1;
                    }

                    fn get_progress(&self) -> f64 {
                        self.cur_step as f64 / self.iteration as f64
                    }
                }

                pub struct SecondProgressScheduler {
                    start_time: f64,
                    seconds: f64,
                }

                impl SecondProgressScheduler {
                    pub fn new(seconds: f64) -> Self {
                        SecondProgressScheduler {
                            start_time: 0.0,
                            seconds,
                        }
                    }
                }

                impl ProgressScheduler for SecondProgressScheduler {
                    fn start(&mut self) {
                        self.start_time = time::elapsed_seconds();
                    }

                    fn get_progress(&self) -> f64 {
                        (time::elapsed_seconds() - self.start_time) / self.seconds
                    }
                }
            }
        }

        pub mod scheduler {
            use crate::ahc_library::{
                annealer::{
                    components::criterion::AnnealingCriterion,
                    components::progress_scheduler::{
                        IterationProgressScheduler, SecondProgressScheduler,
                    },
                    components::temperature_scheduler::ExpTemperatureScheduler,
                    types::{Criterion, ProgressScheduler, TemperatureScheduler},
                },
                utils::random::Random,
            };

            enum AnnealerSchedulerStatus {
                NotStarted,
                InProgress,
                Finished,
            }

            /// Scheduler used for annealing process
            ///
            /// Usage:
            /// ```ignore
            /// let mut scheduler = AnnealerScheduler::with_seconds(1e0, 1e-4, 1.0, true);
            /// while scheduler.to_next_iter() {
            ///     let cur_score = state.get_score();
            ///
            ///     // do something
            ///
            ///     let new_score = state.get_score();
            ///
            ///     if scheduler.adopt(cur_score, new_score) {
            ///         // adopt
            ///     } else {
            ///         // revert
            ///     }
            /// }
            /// ```
            pub struct AnnealerScheduler<C, T, P, R>
            where
                C: Criterion,
                T: TemperatureScheduler,
                P: ProgressScheduler,
                R: Random,
            {
                status: AnnealerSchedulerStatus,
                criterion: C,
                temperature_scheduler: T,
                progress_scheduler: P,
                rnd: R,
            }

            impl<C, T, P, R> AnnealerScheduler<C, T, P, R>
            where
                C: Criterion,
                T: TemperatureScheduler,
                P: ProgressScheduler,
                R: Random,
            {
                pub fn new(
                    criterion: C,
                    temperature_scheduler: T,
                    progress_scheduler: P,
                    rnd: R,
                ) -> Self {
                    AnnealerScheduler {
                        status: AnnealerSchedulerStatus::NotStarted,
                        criterion,
                        temperature_scheduler,
                        progress_scheduler,
                        rnd,
                    }
                }

                pub fn adopt(&mut self, cur_score: f64, new_score: f64) -> bool {
                    let progress = self.get_progress();
                    let cur_temp = self.temperature_scheduler.get_temp(progress);
                    self.criterion
                        .adopt(cur_score, new_score, cur_temp, progress, &mut self.rnd)
                }

                pub fn get_progress(&self) -> f64 {
                    match self.status {
                        AnnealerSchedulerStatus::NotStarted => {
                            panic!("Scheduler has not been started yet.")
                        }
                        AnnealerSchedulerStatus::InProgress => {
                            self.progress_scheduler.get_progress()
                        }
                        AnnealerSchedulerStatus::Finished => 1.,
                    }
                }

                pub fn to_next_iter(&mut self) -> bool {
                    self.status = match self.status {
                        AnnealerSchedulerStatus::NotStarted => {
                            self.progress_scheduler.start();
                            AnnealerSchedulerStatus::InProgress
                        }
                        AnnealerSchedulerStatus::InProgress => {
                            self.progress_scheduler.step();
                            if self.progress_scheduler.get_progress() >= 1. {
                                AnnealerSchedulerStatus::Finished
                            } else {
                                AnnealerSchedulerStatus::InProgress
                            }
                        }
                        AnnealerSchedulerStatus::Finished => AnnealerSchedulerStatus::Finished,
                    };

                    matches!(self.status, AnnealerSchedulerStatus::InProgress)
                }
            }

            impl<R: Random>
                AnnealerScheduler<
                    AnnealingCriterion,
                    ExpTemperatureScheduler,
                    SecondProgressScheduler,
                    R,
                >
            {
                pub fn with_seconds(
                    start_temp: f64,
                    end_temp: f64,
                    seconds: f64,
                    is_maximize: bool,
                    rnd: R,
                ) -> AnnealerScheduler<
                    AnnealingCriterion,
                    ExpTemperatureScheduler,
                    SecondProgressScheduler,
                    R,
                > {
                    AnnealerScheduler::new(
                        AnnealingCriterion::new(is_maximize),
                        ExpTemperatureScheduler::new(start_temp, end_temp),
                        SecondProgressScheduler::new(seconds),
                        rnd,
                    )
                }
            }

            impl<R: Random>
                AnnealerScheduler<
                    AnnealingCriterion,
                    ExpTemperatureScheduler,
                    IterationProgressScheduler,
                    R,
                >
            {
                pub fn with_iterations(
                    start_temp: f64,
                    end_temp: f64,
                    iteration: usize,
                    is_maximize: bool,
                    rnd: R,
                ) -> AnnealerScheduler<
                    AnnealingCriterion,
                    ExpTemperatureScheduler,
                    IterationProgressScheduler,
                    R,
                > {
                    AnnealerScheduler::new(
                        AnnealingCriterion::new(is_maximize),
                        ExpTemperatureScheduler::new(start_temp, end_temp),
                        IterationProgressScheduler::new(iteration),
                        rnd,
                    )
                }
            }
        }

        pub mod types {
            use crate::ahc_library::utils::random::Random;

            pub trait Criterion {
                fn adopt(
                    &self,
                    cur_score: f64,
                    new_score: f64,
                    cur_temp: f64,
                    progress: f64,
                    rnd: &mut impl Random,
                ) -> bool;
            }

            pub trait TemperatureScheduler {
                fn get_temp(&self, progress: f64) -> f64;
            }

            pub trait ProgressScheduler {
                fn start(&mut self) {}
                fn step(&mut self) {}
                fn get_progress(&self) -> f64;
            }
        }
    }

    pub mod utils {
        pub mod dump {
            use num_traits::Num;

            use crate::ahc_library::utils::env::env_is_one;

            pub const AHC_DUMP_ENABLED: bool = env_is_one(option_env!("AHC_DUMP"));

            pub const ANSI_RESET: &str = "\x1b[0m";
            pub const ANSI_BOLD: &str = "\x1b[1m";

            /// `eprintln!` + ANSI style (`ColoredText` or `Color`).
            /// `dump!` is the same but without the newline. Both macros do nothing if the "dump" feature is disabled.
            ///
            /// Set `AHC_DUMP=1` in the environment to enable dumping.
            ///
            /// Examples:
            /// - `dumpln!("a={}", a);` (no color)
            /// - `dumpln!(RED, "a: {}, b: {}", a, b);` (Color as fg)
            /// - `dumpln!(ColoredText::new().fg(RED).bg(BLUE), "{}", "hello");`
            /// - `dumpln!(ColoredText::new().fg(RED).bold(), "bold!");`
            #[macro_export]
            macro_rules! dumpln {
			    // no color
			    ($fmt:literal $(, $arg:expr)*) => {
			        if const { $crate::ahc_library::utils::dump::AHC_DUMP_ENABLED } {
			            eprintln!($fmt $(, $arg)*);
			        }
			    };
			    // with style (Color or ColoredText)
			    ($style:expr, $fmt:literal $(, $arg:expr)*) => {{
			        if const { $crate::ahc_library::utils::dump::AHC_DUMP_ENABLED } {
			            let __style: $crate::ahc_library::utils::dump::ColoredText = $style.into();
			            let __prefix = $crate::ahc_library::utils::dump::ansi_prefix(__style);
			            eprintln!("{}{}{}", __prefix, format_args!($fmt $(, $arg)*), $crate::ahc_library::utils::dump::ANSI_RESET);
			        }
			    }};
			}

            #[macro_export]
            macro_rules! dump {
			    // no color
			    ($fmt:literal $(, $arg:expr)*) => {{
			        if const { $crate::ahc_library::utils::dump::AHC_DUMP_ENABLED } {
			            eprint!($fmt $(, $arg)*);
			        }
			    }};
			    // with style (Color or ColoredText)
			    ($style:expr, $fmt:literal $(, $arg:expr)*) => {{
			        if const { $crate::ahc_library::utils::dump::AHC_DUMP_ENABLED } {
			            let __style: $crate::ahc_library::utils::dump::ColoredText = $style.into();
			            let __prefix = $crate::ahc_library::utils::dump::ansi_prefix(__style);
			            eprint!("{}{}{}", __prefix, format_args!($fmt $(, $arg)*), $crate::ahc_library::utils::dump::ANSI_RESET);
			        }
			    }};
			}

            #[derive(Clone, Copy, Debug, PartialEq)]
            pub struct Color {
                pub r: f64,
                pub g: f64,
                pub b: f64,
                pub br: f64,
            }

            pub mod color {
                use super::Color;

                pub const RED: Color = Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    br: 1.0,
                };
                pub const GREEN: Color = Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    br: 1.0,
                };
                pub const BLUE: Color = Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    br: 1.0,
                };
                pub const WHITE: Color = Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    br: 1.0,
                };
                pub const BLACK: Color = Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    br: 1.0,
                };
                pub const YELLOW: Color = Color {
                    r: 1.0,
                    g: 1.0,
                    b: 0.0,
                    br: 1.0,
                };
                pub const CYAN: Color = Color {
                    r: 0.0,
                    g: 1.0,
                    b: 1.0,
                    br: 1.0,
                };
                pub const MAGENTA: Color = Color {
                    r: 1.0,
                    g: 0.0,
                    b: 1.0,
                    br: 1.0,
                };
            }

            impl Default for Color {
                fn default() -> Self {
                    color::BLACK
                }
            }

            impl Color {
                pub fn new(r: f64, g: f64, b: f64) -> Self {
                    Self { r, g, b, br: 1.0 }
                }

                pub fn with_br(self, br: f64) -> Self {
                    Self { br, ..self }
                }

                /// Convert to (r, g, b) in 0..=255 (without br).
                pub fn to_rgb8(self) -> (u8, u8, u8) {
                    let to_u8 = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    (to_u8(self.r), to_u8(self.g), to_u8(self.b))
                }

                /// Linearly interpolate between `self` (t=0) and `other` (t=1).
                pub fn lerp(self, other: Color, t: f64) -> Color {
                    let t = t.clamp(0.0, 1.0);
                    Color {
                        r: self.r + (other.r - self.r) * t,
                        g: self.g + (other.g - self.g) * t,
                        b: self.b + (other.b - self.b) * t,
                        br: self.br + (other.br - self.br) * t,
                    }
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq)]
            pub struct ColoredText {
                pub fg: Option<Color>,
                pub bg: Option<Color>,
                pub is_bold: bool,
            }

            impl ColoredText {
                pub fn new() -> Self {
                    Self {
                        fg: None,
                        bg: None,
                        is_bold: false,
                    }
                }

                pub fn fg(self, color: Color) -> Self {
                    Self {
                        fg: Some(color),
                        ..self
                    }
                }

                pub fn bg(self, color: Color) -> Self {
                    Self {
                        bg: Some(color),
                        ..self
                    }
                }

                pub fn bold(self) -> Self {
                    Self {
                        is_bold: true,
                        ..self
                    }
                }
            }

            impl Default for ColoredText {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl From<Color> for ColoredText {
                fn from(color: Color) -> Self {
                    Self {
                        fg: Some(color),
                        ..Self::new()
                    }
                }
            }

            /// ANSI truecolor foreground escape sequence with br applied.
            pub fn ansi_fg(color: Color) -> String {
                let b = color.br.clamp(0.0, 1.0);
                let to_u8 = |v: f64| (v.clamp(0.0, 1.0) * b * 255.0).round() as u8;
                format!(
                    "\x1b[38;2;{};{};{}m",
                    to_u8(color.r),
                    to_u8(color.g),
                    to_u8(color.b)
                )
            }

            /// ANSI truecolor background escape sequence with br applied.
            pub fn ansi_bg(color: Color) -> String {
                let b = color.br.clamp(0.0, 1.0);
                let to_u8 = |v: f64| (v.clamp(0.0, 1.0) * b * 255.0).round() as u8;
                format!(
                    "\x1b[48;2;{};{};{}m",
                    to_u8(color.r),
                    to_u8(color.g),
                    to_u8(color.b)
                )
            }

            pub fn ansi_prefix(style: ColoredText) -> String {
                let mut s = String::new();
                if style.is_bold {
                    s.push_str(ANSI_BOLD);
                }
                if let Some(fg) = style.fg {
                    s.push_str(&ansi_fg(fg));
                }
                if let Some(bg) = style.bg {
                    s.push_str(&ansi_bg(bg));
                }
                s
            }

            /// Dump a 2D matrix to stderr, coloring each cell by its normalized value.
            ///
            /// Each value is normalized to [0, 1] over the whole matrix, then the fg color
            /// is interpolated from `low_color` (min) to `high_color` (max).
            pub fn dump_2d<T>(v: &[impl AsRef<[T]>], low_color: Color, high_color: Color)
            where
                T: std::fmt::Display + Copy + Num + num_traits::ToPrimitive + PartialOrd,
            {
                // Collect all values as f64 to find min/max.
                let vals: Vec<f64> = v
                    .iter()
                    .flat_map(|row| row.as_ref().iter())
                    .filter_map(|x| x.to_f64())
                    .collect();

                let min_val = vals.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_val = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let range = max_val - min_val;

                for row in v {
                    for val in row.as_ref() {
                        let f = val.to_f64().unwrap_or(0.0);
                        let t = if range > 0.0 {
                            (f - min_val) / range
                        } else {
                            0.5
                        };
                        let color = low_color.lerp(high_color, t);
                        let style = ColoredText::new().fg(color);
                        dump!(style, "{:>4} ", val);
                    }
                    eprintln!();
                }
            }
        }

        pub mod env {
            pub const fn env_is_one(opt: Option<&str>) -> bool {
                match opt {
                    Some(s) => {
                        let b = s.as_bytes();
                        b.len() == 1 && b[0] == b'1'
                    }
                    None => false,
                }
            }
        }

        pub mod fast_clear_array {
            use crate::ahc_library::utils::ndarray::{Array2d, Array3d};

            pub struct FastClearArray<T: Clone + Copy> {
                pub version: usize,
                pub values: Vec<(usize, T)>,
                pub init_value: T,
            }

            impl<T: Clone + Copy> FastClearArray<T> {
                pub fn new(n: usize, init_value: T) -> FastClearArray<T> {
                    FastClearArray {
                        version: 0,
                        values: vec![(!0, init_value); n],
                        init_value,
                    }
                }

                #[inline]
                pub fn get(&mut self, i: usize) -> T {
                    if self.values[i].0 != self.version {
                        self.values[i] = (self.version, self.init_value);
                    }
                    self.values[i].1
                }

                #[inline]
                pub fn set(&mut self, i: usize, new_value: T) {
                    self.values[i] = (self.version, new_value);
                }

                pub fn clear(&mut self) {
                    self.version += 1;
                }
            }

            #[derive(Clone, Debug)]
            pub struct FastClearArray2d<T: Clone + Copy> {
                pub version: usize,
                pub values: Array2d<(usize, T)>,
                pub init_value: T,
            }

            impl<T: Clone + Copy> FastClearArray2d<T> {
                pub fn new(h: usize, w: usize, init_value: T) -> FastClearArray2d<T> {
                    FastClearArray2d {
                        version: 0,
                        values: Array2d::new(vec![vec![(!0, init_value); w]; h]),
                        init_value,
                    }
                }

                #[inline]
                pub fn get(&mut self, c: &(usize, usize)) -> T {
                    if self.values[*c].0 != self.version {
                        self.values[*c] = (self.version, self.init_value);
                    }
                    self.values[*c].1
                }

                #[inline]
                pub fn set(&mut self, c: &(usize, usize), new_value: T) {
                    self.values[*c] = (self.version, new_value);
                }

                pub fn clear(&mut self) {
                    self.version += 1;
                }
            }

            #[derive(Clone, Debug)]
            pub struct FastClearArray3d<T: Clone + Copy> {
                pub version: usize,
                pub values: Array3d<(usize, T)>,
                pub init_value: T,
            }

            impl<T: Clone + Copy> FastClearArray3d<T> {
                pub fn new(d0: usize, d1: usize, d2: usize, init_value: T) -> FastClearArray3d<T> {
                    FastClearArray3d {
                        version: 0,
                        values: Array3d::init(d0, d1, d2, (0, init_value)),
                        init_value,
                    }
                }

                #[inline]
                pub fn get(&mut self, c: &(usize, usize, usize)) -> T {
                    if self.values[*c].0 != self.version {
                        self.values[*c] = (self.version, self.init_value);
                    }
                    self.values[*c].1
                }

                #[inline]
                pub fn set(&mut self, c: &(usize, usize, usize), new_value: T) {
                    self.values[*c] = (self.version, new_value);
                }

                pub fn clear(&mut self) {
                    self.version += 1;
                }
            }
        }

        pub mod grid {
            const N: usize = 1 << 9;

            /// 3×3 マス目上の関節点（除去すると連結でなくなる頂点）を高速に判定する構造体。
            ///
            /// 3×3 グリッドの部分集合を 9 ビットのビットマスクで表し、
            /// 事前に全 512 通りの連結性を計算したルックアップテーブルを用いる。
            /// これにより関節点の判定を O(1) で行える。
            ///
            /// # ビットマスクの形式
            ///
            /// インデックス `i` のビット（`1 << i`）は、3×3 グリッドを左上から
            /// 行優先で並べたときの位置 `i`（0〜8）のマスが存在することを表す：
            ///
            /// ```text
            /// 0 1 2
            /// 3 4 5
            /// 6 7 8
            /// ```
            #[derive(Debug, Clone)]
            pub struct EasyArticulationChecker {
                connected: [bool; N],
            }

            impl EasyArticulationChecker {
                pub fn new() -> Self {
                    Self {
                        connected: Self::make_connected(),
                    }
                }

                /// 中心マスを追加した時に連結になるか判定する
                /// `grid`は中心マス周辺の8マスのビットマスク（0〜255）で与える
                /// 中心マスに対応するマス（=4bit目）は0である必要がある
                pub fn is_articulation(&self, grid: usize) -> bool {
                    debug_assert!(
                        grid & (1 << 4) == 0,
                        "{:09b} is not a valid grid mask",
                        grid
                    );
                    self.connected[grid | (1 << 4)] && !self.connected[grid]
                }

                fn make_connected() -> [bool; N] {
                    let mut edges = vec![Vec::new(); 9];
                    for v in 0..9 {
                        if v / 3 > 0 {
                            edges[v].push(v - 3);
                        }
                        if v % 3 > 0 {
                            edges[v].push(v - 1);
                        }
                        if v % 3 < 2 {
                            edges[v].push(v + 1);
                        }
                        if v / 3 < 2 {
                            edges[v].push(v + 3);
                        }
                    }
                    let mut connected = [false; N];
                    connected[0] = true;
                    for s in 1..N {
                        let root = s.trailing_zeros();
                        let mut visited = 1 << root;
                        let mut todo: usize = 1 << root;
                        while todo > 0 {
                            let u = todo.trailing_zeros() as usize;
                            todo ^= 1 << u;
                            for &v in &edges[u] {
                                if (s & (1 << v)) > 0 && (visited & (1 << v)) == 0 {
                                    visited |= 1 << v;
                                    todo |= 1 << v;
                                }
                            }
                        }
                        if visited == s {
                            connected[s] = true;
                        }
                    }
                    connected
                }
            }
        }

        pub mod index_set {
            use crate::ahc_library::utils::random::Random;

            const NONE_POS: usize = !0;

            #[derive(Debug, Clone)]
            pub struct IndexSet {
                pub que: Vec<usize>,
                pub pos: Vec<usize>,
            }

            impl IndexSet {
                pub fn empty(n: usize) -> Self {
                    IndexSet {
                        que: Vec::with_capacity(n),
                        pos: vec![NONE_POS; n],
                    }
                }

                pub fn full(n: usize) -> Self {
                    IndexSet {
                        que: (0..n).collect(),
                        pos: (0..n).collect(),
                    }
                }

                pub fn clear(&mut self) {
                    for &v in &self.que {
                        self.pos[v] = NONE_POS;
                    }
                    self.que.clear();
                }

                pub fn add(&mut self, v: usize) {
                    if self.contains(v) {
                        return;
                    }
                    self.pos[v] = self.que.len();
                    self.que.push(v);
                }

                pub fn remove(&mut self, v: usize) {
                    if !self.contains(v) {
                        return;
                    }

                    let p = self.pos[v];
                    let b = self.que[self.que.len() - 1];
                    self.que.swap_remove(p);
                    self.pos[b] = p;
                    self.pos[v] = NONE_POS;
                }

                pub fn contains(&self, v: usize) -> bool {
                    self.pos[v] != NONE_POS
                }

                pub fn size(&self) -> usize {
                    self.que.len()
                }

                pub fn first(&self) -> Option<usize> {
                    self.que.get(0).copied()
                }

                pub fn get_random(&self, rnd: &mut impl Random) -> Option<usize> {
                    self.que.get(rnd.gen_index(self.que.len())).copied()
                }

                pub fn iter(&self) -> impl Iterator<Item = &usize> {
                    self.que.iter()
                }
            }

            #[derive(Debug, Clone)]
            pub struct IndexMap<T> {
                set: IndexSet,
                vals: Vec<Option<T>>,
            }

            impl<T> IndexMap<T>
            where
                T: Clone + Copy + Default,
            {
                pub fn new(n: usize) -> Self {
                    IndexMap {
                        set: IndexSet::empty(n),
                        vals: vec![None; n],
                    }
                }

                pub fn add(&mut self, idx: usize, val: T) {
                    if !self.set.contains(idx) {
                        self.set.add(idx);
                    }
                    self.vals[idx] = Some(val);
                }

                pub fn remove(&mut self, idx: usize) {
                    if self.set.contains(idx) {
                        self.set.remove(idx);
                        self.vals[idx] = None;
                    }
                }

                pub fn get(&self, idx: usize) -> Option<T> {
                    self.vals[idx]
                }

                pub fn iter(&self) -> impl Iterator<Item = (usize, T)> + '_ {
                    self.set.iter().map(move |&idx| {
                        (idx, self.vals[idx].expect("Somehow IndexMap is invalid"))
                    })
                }
            }
        }

        pub mod ndarray {
            use std::ops::{Index, IndexMut};

            use super::v2::V2;

            #[derive(Clone, Debug)]
            pub struct ArrayRef2d<T>
            where
                T: Clone,
            {
                pub h: usize,
                pub w: usize,
                pub values: Vec<T>,
            }

            impl<T> ArrayRef2d<T>
            where
                T: Clone,
            {
                /// expect: values[i].len() = const.
                pub fn new(values: Vec<Vec<T>>) -> ArrayRef2d<T> {
                    let h = values.len();
                    let w = values[0].len();
                    let values = values.into_iter().flatten().collect();
                    ArrayRef2d { h, w, values }
                }

                pub fn init(h: usize, w: usize, init_value: T) -> ArrayRef2d<T> {
                    let values = vec![init_value; h * w];
                    ArrayRef2d { h, w, values }
                }

                #[inline]
                pub fn iter(&self) -> std::slice::Iter<'_, T> {
                    self.values.iter()
                }
            }

            impl<T> From<Vec<Vec<T>>> for ArrayRef2d<T>
            where
                T: Clone,
            {
                fn from(values: Vec<Vec<T>>) -> Self {
                    ArrayRef2d::new(values)
                }
            }

            impl<T> Index<V2<usize>> for ArrayRef2d<T>
            where
                T: Clone,
            {
                type Output = T;

                #[inline]
                fn index(&self, v: V2<usize>) -> &Self::Output {
                    &self.values[v.x * self.w + v.y]
                }
            }

            impl<T> IndexMut<V2<usize>> for ArrayRef2d<T>
            where
                T: Clone,
            {
                #[inline]
                fn index_mut(&mut self, v: V2<usize>) -> &mut Self::Output {
                    &mut self.values[v.x * self.w + v.y]
                }
            }

            impl<T> Index<(usize, usize)> for ArrayRef2d<T>
            where
                T: Clone,
            {
                type Output = T;

                #[inline]
                fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
                    &self.values[x * self.w + y]
                }
            }

            impl<T> IndexMut<(usize, usize)> for ArrayRef2d<T>
            where
                T: Clone,
            {
                #[inline]
                fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
                    &mut self.values[x * self.w + y]
                }
            }

            /// A 2D array stored in a 1D Vec.
            /// The element type T must implement Clone and Copy.
            ///
            /// # Example
            /// ```
            /// use ahc_library::utils::ndarray::Array2d;
            /// let mut array = Array2d::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
            /// assert_eq!(array[(0, 0)], 1);
            /// assert_eq!(array[(1, 2)], 6);
            /// array[(0, 1)] = 10;
            /// assert_eq!(array[(0, 1)], 10);
            /// ```
            ///
            /// You can use V2<usize> as index as well:
            /// ```
            /// use ahc_library::utils::{ndarray::Array2d, v2::V2};
            /// let mut array = Array2d::new(vec![vec![1, 2, 3], vec![4, 5, 6]]);
            /// let v = V2::new(1, 0);
            /// assert_eq!(array[v], 4);
            /// array[v] = 20;
            /// assert_eq!(array[v], 20);
            /// ```
            #[derive(Clone, Debug)]
            pub struct Array2d<T>
            where
                T: Clone + Copy,
            {
                pub h: usize,
                pub w: usize,
                values: Vec<T>,
            }

            impl<T> Array2d<T>
            where
                T: Clone + Copy,
            {
                /// expect: values[i].len() = const.
                pub fn new(values: Vec<Vec<T>>) -> Array2d<T> {
                    let h = values.len();
                    let w = values[0].len();
                    let values = values.into_iter().flatten().collect();
                    Array2d { h, w, values }
                }

                pub fn init(h: usize, w: usize, init_value: T) -> Array2d<T> {
                    let values = vec![init_value; h * w];
                    Array2d { h, w, values }
                }

                #[inline]
                pub fn iter(&self) -> std::slice::Iter<'_, T> {
                    self.values.iter()
                }
            }

            impl<T> From<Vec<Vec<T>>> for Array2d<T>
            where
                T: Clone + Copy,
            {
                fn from(values: Vec<Vec<T>>) -> Self {
                    Array2d::new(values)
                }
            }

            impl<T> Index<V2<usize>> for Array2d<T>
            where
                T: Clone + Copy,
            {
                type Output = T;

                #[inline]
                fn index(&self, v: V2<usize>) -> &Self::Output {
                    &self.values[v.x * self.w + v.y]
                }
            }

            impl<T> IndexMut<V2<usize>> for Array2d<T>
            where
                T: Clone + Copy,
            {
                #[inline]
                fn index_mut(&mut self, v: V2<usize>) -> &mut Self::Output {
                    &mut self.values[v.x * self.w + v.y]
                }
            }

            impl<T> Index<(usize, usize)> for Array2d<T>
            where
                T: Clone + Copy,
            {
                type Output = T;

                #[inline]
                fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
                    &self.values[x * self.w + y]
                }
            }

            impl<T> IndexMut<(usize, usize)> for Array2d<T>
            where
                T: Clone + Copy,
            {
                #[inline]
                fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
                    &mut self.values[x * self.w + y]
                }
            }

            #[derive(Clone, Debug)]
            pub struct Array3d<T>
            where
                T: Clone + Copy,
            {
                pub d0: usize,
                pub d1: usize,
                pub d2: usize,
                values: Vec<T>,
            }

            impl<T> Array3d<T>
            where
                T: Clone + Copy,
            {
                pub fn init(d0: usize, d1: usize, d2: usize, init_value: T) -> Array3d<T> {
                    let values = vec![init_value; d0 * d1 * d2];
                    Array3d { d0, d1, d2, values }
                }
            }

            impl<T> Index<(usize, usize, usize)> for Array3d<T>
            where
                T: Clone + Copy,
            {
                type Output = T;

                #[inline]
                fn index(&self, (x, y, z): (usize, usize, usize)) -> &Self::Output {
                    &self.values[x * self.d1 * self.d2 + y * self.d2 + z]
                }
            }

            impl<T> IndexMut<(usize, usize, usize)> for Array3d<T>
            where
                T: Clone + Copy,
            {
                #[inline]
                fn index_mut(&mut self, (x, y, z): (usize, usize, usize)) -> &mut Self::Output {
                    &mut self.values[x * self.d1 * self.d2 + y * self.d2 + z]
                }
            }
        }

        pub mod object_pool {
            use crate::ahc_library::utils::index_set::IndexSet;

            /// プール可能なオブジェクトのトレイト
            pub trait Poolable: Clone {
                fn new() -> Self;
                fn reset(&mut self);
            }

            #[derive(Clone, Debug)]
            pub struct ObjectPool<T: Poolable> {
                objects: Vec<Option<T>>,
                can_use_slots: IndexSet,
                empty_slots: IndexSet,
            }

            impl<T: Poolable> ObjectPool<T> {
                pub fn new(pool_size: usize) -> ObjectPool<T> {
                    ObjectPool {
                        objects: vec![None; pool_size],
                        can_use_slots: IndexSet::full(pool_size),
                        empty_slots: IndexSet::empty(pool_size),
                    }
                }

                /// オブジェクトをプールに戻す
                pub fn pool(&mut self, obj: T) -> Option<usize> {
                    if let Some(idx) = self.empty_slots.first() {
                        self.objects[idx] = Some(obj);
                        self.empty_slots.remove(idx);
                        self.can_use_slots.add(idx);
                        Some(idx)
                    } else {
                        None
                    }
                }

                /// プールからオブジェクトを取得する
                pub fn get_new(&mut self) -> Option<T> {
                    if let Some(idx) = self.can_use_slots.first() {
                        self.can_use_slots.remove(idx);
                        self.empty_slots.add(idx);
                        let mut obj = self.objects[idx].take().unwrap_or(T::new());
                        obj.reset();
                        Some(obj)
                    } else {
                        None
                    }
                }

                pub fn n_remain(&self) -> usize {
                    self.can_use_slots.size()
                }
            }
        }

        pub mod param {
            /// `Params` 構造体と、その読み込み処理を定義するマクロ
            ///
            /// ```ignore
            /// params_impl! {
            ///     { START_TEMP: f64, END_TEMP: f64 },
            ///     [
            ///         "group_0" => { START_TEMP: 1000.0, END_TEMP: 10.0 },
            ///         "group_1" => { START_TEMP: 5000.0, END_TEMP: 100.0 },
            ///         _ => { START_TEMP: 2000.0, END_TEMP: 20.0 },
            ///     ]
            /// }
            ///
            /// let params = Params::load("group_0");
            /// assert_eq!(params.START_TEMP, 1000.0);
            /// ```
            #[macro_export]
            macro_rules! params_impl {
			    (
			        { $( $pname:ident : $pty:ty ),* $(,)? },
			        [ $( $pat:pat => { $( $fname:ident : $fval:expr ),* $(,)? } ),+ $(,)? ]
			    ) => {
			        #[allow(non_snake_case, unused)]
			        #[derive(Debug, Clone)]
			        pub struct Params {
			            $( pub $pname: $pty, )*
			        }

			        impl Params {
			            pub fn load(group_id: &str) -> Self {
			                match group_id {
			                    $(
			                        $pat => Self {
			                            $( $fname: $fval, )*
			                        },
			                    )*
			                }
			            }
			        }
			    };

			    (
			        $(
			            $name:ident: $type:ty = $default:expr
			        ),* $(,)?
			    ) => {
			        #[allow(non_snake_case, unused)]
			        #[derive(Debug, Clone)]
			        pub struct Params {
			            $(
			                pub $name: $type,
			            )*
			        }

			        impl Params {
			            fn load() -> Self {
			                Self {
			                    $(
			                        $name: std::env::var(stringify!($name))
			                            .ok()
			                            .map(|v| v.parse::<$type>().unwrap())
			                            .unwrap_or($default),
			                    )*
			                }
			            }
			        }
			    };
			}
        }

        pub mod path_finder {
            use std::collections::VecDeque;

            use crate::ahc_library::utils::{
                fast_clear_array::FastClearArray2d, ndarray::Array2d, random::Random, v2::Coor,
            };

            pub trait PathFindState {
                fn trans(&self, u: &Coor<usize>, path: &[Coor<usize>]) -> Self;
            }

            #[derive(Clone, Copy)]
            pub struct DummyPathFindState;

            impl PathFindState for DummyPathFindState {
                fn trans(&self, _: &Coor<usize>, _: &[Coor<usize>]) -> Self {
                    *self
                }
            }

            pub struct BfsGridPathFinder<R: Random> {
                h: usize,
                w: usize,
                q: VecDeque<Coor<usize>>,
                dist: FastClearArray2d<i32>,
                prev: FastClearArray2d<Option<Coor<usize>>>,
                seen: Vec<Coor<usize>>,
                rnd: R,
            }

            impl<R: Random> BfsGridPathFinder<R> {
                pub fn new(h: usize, w: usize, rnd: R) -> Self {
                    Self {
                        h,
                        w,
                        q: VecDeque::new(),
                        dist: FastClearArray2d::new(h, w, i32::MAX),
                        prev: FastClearArray2d::new(h, w, None),
                        seen: Vec::with_capacity(h * w),
                        rnd,
                    }
                }

                pub fn get_reachable_coors<T, D>(
                    &mut self,
                    start: &Coor<usize>,
                    trans_cond: T,
                    priority_d: D,
                ) -> Vec<Coor<usize>>
                where
                    T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
                    D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
                {
                    self._bfs(start, |_| false, trans_cond, priority_d);
                    self.seen.clone()
                }

                pub fn get_reachable_size<T, D>(
                    &mut self,
                    start: &Coor<usize>,
                    trans_cond: T,
                    priority_d: D,
                ) -> usize
                where
                    T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
                    D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
                {
                    self._bfs(start, |_| false, trans_cond, priority_d);
                    self.seen.len()
                }

                /// 両端点を含む
                pub fn find_path<C, T, D>(
                    &mut self,
                    start: &Coor<usize>,
                    complete_cond: C,
                    trans_cond: T,
                    priority_d: D,
                ) -> Option<Vec<Coor<usize>>>
                where
                    C: Fn(&Coor<usize>) -> bool,
                    T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
                    D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
                {
                    let v = self._bfs(start, complete_cond, trans_cond, priority_d)?;
                    Some(self.restore_path(start, &v))
                }

                fn _bfs<C, T, D>(
                    &mut self,
                    start: &Coor<usize>,
                    complete_cond: C,
                    trans_cond: T,
                    priority_d: D,
                ) -> Option<Coor<usize>>
                where
                    C: Fn(&Coor<usize>) -> bool,
                    T: Fn(&Coor<usize>, &Coor<usize>) -> bool,
                    D: Fn(usize, &Coor<usize>, &mut R) -> Coor<usize>,
                {
                    self.reset();

                    self.dist.set(&(start.i, start.j), 0);
                    self.q.push_back(*start);
                    self.seen.push(*start);

                    while let Some(v) = self.q.pop_front() {
                        if complete_cond(&v) {
                            return Some(v);
                        }

                        let new_dist = self.dist.get(&(v.i, v.j)) + 1;
                        for i in 0..4 {
                            let d = priority_d(i, &v, &mut self.rnd);
                            let u = v + d;
                            if u.i < self.h
                                && u.j < self.w
                                && (trans_cond)(&u, &v)
                                && new_dist < self.dist.get(&(u.i, u.j))
                            {
                                self.q.push_back(u);

                                self.dist.set(&(u.i, u.j), new_dist);
                                self.prev.set(&(u.i, u.j), Some(v));
                                self.seen.push(u);
                            }
                        }
                    }

                    None
                }

                pub fn restore_path(
                    &mut self,
                    start: &Coor<usize>,
                    end: &Coor<usize>,
                ) -> Vec<Coor<usize>> {
                    let mut path = vec![*end];
                    let mut cur = *end;
                    while let Some(p) = self.prev.get(&(cur.i, cur.j)) {
                        cur = p;
                        path.push(cur);
                    }
                    path.reverse();

                    assert_eq!(&cur, start);

                    path
                }

                pub fn get_dist_table(&mut self) -> Array2d<usize> {
                    let mut array2d = Array2d::init(self.h, self.w, 0);
                    for i in 0..self.h {
                        for j in 0..self.w {
                            array2d[(i, j)] = self.dist.get(&(i, j)) as usize;
                        }
                    }
                    array2d
                }

                fn reset(&mut self) {
                    self.dist.clear();
                    self.q.clear();
                    self.prev.clear();
                    self.seen.clear();
                }
            }
        }

        pub mod random {
            use rand_pcg::rand_core::{RngCore, SeedableRng};

            pub trait Random {
                fn _next(&mut self) -> u32;

                #[inline(always)]
                fn nextf(&mut self) -> f64 {
                    self._next() as f64 / ((1u64 << 32) as f64)
                }

                #[inline(always)]
                fn choice<T: Clone + Copy>(&mut self, v: &[T]) -> T {
                    let idx = self.gen_index(v.len());
                    v[idx]
                }

                #[inline(always)]
                fn gen_index(&mut self, len: usize) -> usize {
                    debug_assert!(len as u64 <= 1 << 32);
                    ((len as u64 * self._next() as u64) >> 32) as usize
                }

                #[inline(always)]
                fn gen_range(&mut self, l: usize, r: usize) -> usize {
                    debug_assert!(l < r);
                    debug_assert!(r as u64 <= 1 << 32);
                    l + (((r - l) as u64 * self._next() as u64) >> 32) as usize
                }

                #[inline(always)]
                fn gen_rangef(&mut self, l: f64, r: f64) -> f64 {
                    debug_assert!(l <= r);
                    l + self._next() as f64 * ((r - l) / ((1u64 << 32) as f64))
                }

                #[inline(always)]
                fn shuffle<T>(&mut self, v: &mut [T]) {
                    let n = v.len();
                    for i in (1..n).rev() {
                        let j = self.gen_range(0, i + 1);
                        v.swap(i, j);
                    }
                }
            }

            #[derive(Debug, Clone, Copy)]
            pub struct XorShift32 {
                state: u32,
            }

            impl XorShift32 {
                pub fn new(seed: u32) -> Self {
                    assert_ne!(seed, 0);
                    Self { state: seed }
                }
            }

            impl Random for XorShift32 {
                #[inline(always)]
                fn _next(&mut self) -> u32 {
                    let mut x = self.state;
                    x ^= x << 13;
                    x ^= x >> 17;
                    x ^= x << 5;
                    self.state = x;
                    x
                }
            }

            #[derive(Debug, Clone)]
            pub struct RandPcg64Mcg {
                inner: rand_pcg::Pcg64Mcg,
            }

            impl RandPcg64Mcg {
                pub fn new(seed: u64) -> Self {
                    Self {
                        inner: rand_pcg::Pcg64Mcg::seed_from_u64(seed),
                    }
                }
            }

            impl Random for RandPcg64Mcg {
                #[inline(always)]
                fn _next(&mut self) -> u32 {
                    self.inner.next_u32()
                }
            }

            #[derive(Debug, Clone)]
            pub struct BufferedRandom {
                buf: Vec<u32>,
                pos: usize,
            }

            impl BufferedRandom {
                pub fn new<R: Random>(rnd: &mut R, buf_size: usize) -> Self {
                    assert!(0 < buf_size);
                    assert!(buf_size < 1_000_000);
                    let mut buf = Vec::with_capacity(buf_size);
                    for _ in 0..buf_size {
                        buf.push(rnd._next());
                    }
                    Self { buf, pos: 0 }
                }
            }

            impl Random for BufferedRandom {
                #[inline(always)]
                fn _next(&mut self) -> u32 {
                    let v = self.buf[self.pos];
                    self.pos += 1;
                    if self.pos == self.buf.len() {
                        self.pos = 0;
                    }
                    v
                }
            }

            pub trait RandomSampler<T> {
                fn sample(&mut self) -> T;
            }

            pub struct DiscreteSampler<T, R> {
                buf: Vec<T>,
                rnd: R,
            }

            impl<T: Copy, R: Random> DiscreteSampler<T, R> {
                pub fn new(weight_values: &Vec<(usize, T)>, rnd: R) -> Self {
                    let weight_sum = weight_values.iter().map(|(w, _)| *w).sum::<usize>();
                    assert!(0 < weight_sum);
                    assert!(weight_sum < 1_000_000);
                    let mut buf = Vec::with_capacity(weight_sum);
                    for &(w, val) in weight_values.iter() {
                        buf.extend(std::iter::repeat(val).take(w));
                    }
                    Self { buf, rnd }
                }
            }

            impl<T: Copy, R: Random> RandomSampler<T> for DiscreteSampler<T, R> {
                fn sample(&mut self) -> T {
                    self.rnd.choice(&self.buf)
                }
            }

            pub struct ContinousSampler<R: Random> {
                buf: Vec<f64>,
                rnd: R,
            }

            impl<R: Random> ContinousSampler<R> {
                pub fn new<F>(f: F, x_min: f64, x_max: f64, size: usize, rnd: R) -> Self
                where
                    F: Fn(f64) -> f64,
                {
                    assert!(0 < size);
                    assert!(size < 1_000_000);
                    let mut buf = Vec::with_capacity(size);
                    let step = (x_max - x_min) / (size as f64 - 1.);
                    for i in 0..size {
                        let x = x_min + step * (i as f64);
                        buf.push(f(x));
                    }
                    Self { buf, rnd }
                }
            }

            impl<R: Random> RandomSampler<f64> for ContinousSampler<R> {
                fn sample(&mut self) -> f64 {
                    self.rnd.choice(&self.buf)
                }
            }
        }

        pub mod stopwatch {
            /// https://github.com/terry-u16/cp-lib-rs/blob/master/src/diagnostics.rs より拝借
            use itertools::Itertools;
            use rustc_hash::FxHashMap;
            use std::{
                borrow::Cow,
                cell::RefCell,
                fmt::Display,
                io::{self, IsTerminal as _},
                rc::Rc,
                time::{Duration, Instant},
            };

            use crate::ahc_library::utils::env::env_is_one;

            pub const AHC_PERF_ENABLED: bool = env_is_one(option_env!("AHC_PERF"));

            /// Set `AHC_PERF=1` in the environment to enable performance measurement.
            #[macro_export]
            macro_rules! perf {
                ($name:expr $(,)?) => {
                    let _sw = if const { $crate::ahc_library::utils::stopwatch::AHC_PERF_ENABLED } {
                        Some(ahc_library::utils::stopwatch::Perf::start_singleton($name))
                    } else {
                        None
                    };
                };
            }

            /// 実行時間を計測・集計する構造体
            ///
            /// # Examples
            ///
            /// ```
            /// use ahc_library::utils::stopwatch::Perf;
            ///
            /// // 計測グループを作成する
            /// // dropされるときに計測結果を出力する
            /// let mut perf = Perf::new("Group");
            /// let mut _sum = 0u64;
            ///
            /// for i in 0..100000 {
            ///     // start-stop間の処理時間を計測する
            ///     let sw = perf.start("sum");
            ///     _sum += i;
            ///     sw.stop();
            ///
            ///     // 名前は&'strでもStringでもOK
            ///     let sw = perf.start(format!("no-op"));
            ///     sw.stop();
            /// }
            ///
            /// let mut _sum_sqrt = 0f64;
            ///
            /// for i in 0..100000 {
            ///     // 明示的にstop()を呼ばなくても、スコープを抜ける際に自動でstopする
            ///     let _sw = perf.start("sum sqrt");
            ///     _sum_sqrt += (i as f64).sqrt();
            /// }
            ///
            /// let mut _sum_sq = 0u64;
            ///
            /// for i in 0..100000 {
            ///     // perfのインスタンス化が面倒な場合はシングルトンを使う
            ///     let sw = Perf::start_singleton("sum sq");
            ///     _sum_sq += i * i;
            ///     sw.stop();
            /// }
            /// ```
            pub struct Perf {
                name: Option<Cow<'static, str>>,
                measures: FxHashMap<Cow<'static, str>, Measure>,
            }

            impl Perf {
                thread_local!(static SINGLETON: Rc<RefCell<Perf>> = Rc::new(RefCell::new(Perf::new("Singleton"))));

                /// 新しい Perf インスタンスを作成する
                pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
                    Self {
                        name: Some(name.into()),
                        measures: FxHashMap::default(),
                    }
                }

                /// 匿名の Perf インスタンスを作成する
                pub fn new_anonymous() -> Self {
                    Self {
                        name: None,
                        measures: FxHashMap::default(),
                    }
                }

                /// 計測を開始する
                pub fn start(
                    &mut self,
                    name: impl Into<Cow<'static, str>>,
                ) -> StopWatch<&mut Perf> {
                    let name = name.into();

                    StopWatch {
                        start: Instant::now(),
                        perf: self,
                        name,
                    }
                }

                /// 計測を開始する（シングルトン）
                pub fn start_singleton(
                    name: impl Into<Cow<'static, str>>,
                ) -> StopWatch<Rc<RefCell<Perf>>> {
                    Self::SINGLETON.with(|perf| StopWatch {
                        start: Instant::now(),
                        perf: perf.clone(),
                        name: name.into(),
                    })
                }
            }

            impl Drop for Perf {
                fn drop(&mut self) {
                    if self.measures.is_empty() {
                        return;
                    }

                    // ターミナルかどうかで色を付けるかどうか分岐
                    let is_tty = io::stderr().is_terminal();
                    let name = self.name.as_deref().unwrap_or("Anonymous");

                    if is_tty {
                        // コンソール → 色付き
                        eprintln!("\x1b[35m[{name}] Performance measures\x1b[0m");
                    } else {
                        // リダイレクト → 色なし
                        eprintln!("[{name}] Performance measures");
                    }

                    for (name, measure) in
                        self.measures.iter().sorted_unstable_by(|(_, ma), (_, mb)| {
                            mb.sum
                                .partial_cmp(&ma.sum)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                    {
                        eprintln!("{name}: {measure}");
                    }
                }
            }

            pub struct StopWatch<M: WithMut<Perf>> {
                start: Instant,
                perf: M,
                name: Cow<'static, str>,
            }

            impl<M: WithMut<Perf>> StopWatch<M> {
                pub fn stop(self) {
                    std::mem::drop(self);
                }
            }

            impl<M: WithMut<Perf>> Drop for StopWatch<M> {
                fn drop(&mut self) {
                    let duration = self.start.elapsed();
                    let name = std::mem::take(&mut self.name);

                    self.perf.with_mut(|perf| {
                        perf.measures
                            .entry(name)
                            .or_default()
                            .add_measure(&duration)
                    })
                }
            }

            #[derive(Debug, Clone, Copy, Default)]
            struct Measure {
                sum: f64,
                sum_sq: f64,
                cnt: usize,
            }

            impl Measure {
                fn add_measure(&mut self, duration: &Duration) {
                    let sec = duration.as_secs_f64();
                    self.sum += sec;
                    self.sum_sq += sec * sec;
                    self.cnt += 1;
                }

                fn mean(&self) -> Duration {
                    assert_ne!(self.cnt, 0);
                    Duration::from_secs_f64(self.sum / self.cnt as f64)
                }

                fn std_dev(&self) -> Duration {
                    assert_ne!(self.cnt, 0);

                    let mean = self.mean().as_secs_f64();
                    let variance = (self.sum_sq / self.cnt as f64) - (mean * mean);
                    Duration::from_secs_f64(variance.sqrt())
                }
            }

            impl Display for Measure {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "{:?}, average: {:?} ± {:?} ({} samples)",
                        Duration::from_secs_f64(self.sum),
                        self.mean(),
                        self.std_dev(),
                        self.cnt
                    )
                }
            }

            /// &mut T と Rc<RefCell<T>> を統一的に扱って処理を行うためのトレイト
            pub trait WithMut<T> {
                fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R;
            }

            impl<T> WithMut<T> for &mut T {
                fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
                    f(*self)
                }
            }

            impl<T> WithMut<T> for Rc<RefCell<T>> {
                fn with_mut<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
                    let mut guard = self.borrow_mut();
                    f(&mut *guard)
                }
            }
        }

        pub mod time {
            static mut START: f64 = -1.;
            static mut R: f64 = 1.;

            #[allow(unused)]
            /// r - scaling factor for elapsed time
            pub fn start_clock(r: f64) {
                unsafe {
                    R = r;
                }
                let _ = elapsed_seconds();
            }

            #[inline]
            #[allow(unused)]
            pub fn elapsed_seconds() -> f64 {
                let t = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                unsafe {
                    if START < 0. {
                        START = t;
                    }
                    (t - START) * R
                }
            }
        }

        pub mod topklist {
            /// https://atcoder.jp/contests/ahc052/submissions/68703195 より拝借
            use std::collections::BinaryHeap;

            #[derive(Clone, Debug)]
            struct Entry<K, V> {
                k: K,
                v: V,
            }

            impl<K: PartialOrd, V> Ord for Entry<K, V> {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.k.partial_cmp(&other.k).unwrap()
                }
            }

            #[allow(clippy::non_canonical_partial_ord_impl)]
            impl<K: PartialOrd, V> PartialOrd for Entry<K, V> {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    self.k.partial_cmp(&other.k)
                }
            }

            impl<K: PartialEq, V> PartialEq for Entry<K, V> {
                fn eq(&self, other: &Self) -> bool {
                    self.k.eq(&other.k)
                }
            }

            impl<K: PartialEq, V> Eq for Entry<K, V> {}

            #[derive(Clone, Debug)]
            pub struct BoundedSortedList<K: PartialOrd + Copy, V: Clone> {
                que: BinaryHeap<Entry<K, V>>,
                size: usize,
            }

            impl<K: PartialOrd + Copy, V: Clone> BoundedSortedList<K, V> {
                pub fn new(size: usize) -> Self {
                    Self {
                        que: BinaryHeap::with_capacity(size),
                        size,
                    }
                }

                pub fn can_insert(&self, k: K) -> bool {
                    self.que.len() < self.size || self.que.peek().unwrap().k > k
                }

                pub fn insert(&mut self, k: K, v: V) {
                    if self.que.len() < self.size {
                        self.que.push(Entry { k, v });
                    } else if let Some(mut top) = self.que.peek_mut() {
                        if top.k > k {
                            top.k = k;
                            top.v = v;
                        }
                    }
                }

                pub fn to_list(self) -> Vec<(K, V)> {
                    let v = self.que.into_sorted_vec();
                    v.into_iter().map(|e| (e.k, e.v)).collect()
                }

                pub fn len(&self) -> usize {
                    self.que.len()
                }

                pub fn is_empty(&self) -> bool {
                    self.que.is_empty()
                }
            }
        }

        pub mod v2 {
            /// 2D Vector that supports basic arithmetic operations.
            /// # Examples
            /// ```
            /// use ahc_library::utils::v2::V2;
            /// let v1 = V2::new(1, 2);
            /// let v2 = V2::new(3, 4);
            /// let v3 = v1 + v2;
            /// assert_eq!(v3.x, 4);
            /// assert_eq!(v3.y, 6);
            ///
            /// let mut v4 = V2::new(5, 6);
            /// v4 += 10;
            /// assert_eq!(v4.x, 15);
            /// assert_eq!(v4.y, 16);
            /// ```
            macro_rules! define_vec2 {
			    (
			        $name:ident,
			        $f1:ident,
			        $f2:ident
			    ) => {
			        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
			        pub struct $name<T>
			        where
			            T: num_traits::Num
			        {
			            pub $f1: T,
			            pub $f2: T,
			        }

			        /* ---------- binary ops ---------- */
			        macro_rules! impl_bin_op {
			            ($Trait:ident, $method:ident, $op:tt) => {
			                impl<T> std::ops::$Trait for $name<T>
			                where
			                    T: num_traits::Num + std::ops::$Trait<Output = T>,
			                {
			                    type Output = Self;
			                    fn $method(self, other: Self) -> Self {
			                        Self {
			                            $f1: self.$f1 $op other.$f1,
			                            $f2: self.$f2 $op other.$f2,
			                        }
			                    }
			                }

			                impl<T> std::ops::$Trait<T> for $name<T>
			                where
			                    T: num_traits::Num
			                        + Copy
			                        + std::ops::Add<Output = T>
			                        + std::ops::Sub<Output = T>
			                        + std::ops::Mul<Output = T>
			                        + std::ops::Div<Output = T>,
			                {
			                    type Output = Self;
			                    fn $method(self, factor: T) -> Self {
			                        Self {
			                            $f1: self.$f1 $op factor,
			                            $f2: self.$f2 $op factor,
			                        }
			                    }
			                }
			            };
			        }

			        macro_rules! impl_assign_op {
			            ($Trait:ident, $method:ident, $op:tt) => {
			                impl<T> std::ops::$Trait for $name<T>
			                where
			                    T: num_traits::Num + std::ops::$Trait,
			                {
			                    fn $method(&mut self, other: Self) {
			                        self.$f1 $op other.$f1;
			                        self.$f2 $op other.$f2;
			                    }
			                }

			                impl<T> std::ops::$Trait<T> for $name<T>
			                where
			                    T: num_traits::Num + std::ops::$Trait + Copy,
			                {
			                    fn $method(&mut self, factor: T) {
			                        self.$f1 $op factor;
			                        self.$f2 $op factor;
			                    }
			                }
			            };
			        }

			        impl_bin_op!(Add, add, +);
			        impl_bin_op!(Sub, sub, -);
			        impl_bin_op!(Mul, mul, *);
			        impl_bin_op!(Div, div, /);

			        impl_assign_op!(AddAssign, add_assign, +=);
			        impl_assign_op!(SubAssign, sub_assign, -=);
			        impl_assign_op!(MulAssign, mul_assign, *=);
			        impl_assign_op!(DivAssign, div_assign, /=);

			        /* ---------- misc ---------- */
			        impl<T> std::fmt::Display for $name<T>
			        where
			            T: num_traits::Num + std::fmt::Display,
			        {
			            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			                write!(f, "({}, {})", self.$f1, self.$f2)
			            }
			        }

			        impl<T> std::fmt::Debug for $name<T>
			        where
			            T: num_traits::Num + std::fmt::Display,
			        {
			            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			                write!(f, "({}, {})", self.$f1, self.$f2)
			            }
			        }

			        impl<T> num_traits::Zero for $name<T>
			        where
			            T: num_traits::Num,
			        {
			            fn zero() -> Self {
			                Self {
			                    $f1: T::zero(),
			                    $f2: T::zero(),
			                }
			            }

			            fn is_zero(&self) -> bool {
			                self.$f1.is_zero() && self.$f2.is_zero()
			            }
			        }

			        impl<T> $name<T>
			        where
			            T: num_traits::Num,
			        {
			            pub fn new($f1: T, $f2: T) -> Self {
			                Self { $f1, $f2 }
			            }
			        }
			    };
			}

            define_vec2!(V2, x, y);
            define_vec2!(Coor, i, j);

            /// Coor utilities
            pub const D_DOWN: Coor<usize> = Coor { i: 1, j: 0 };
            pub const D_UP: Coor<usize> = Coor { i: !0, j: 0 };
            pub const D_LEFT: Coor<usize> = Coor { i: 0, j: !0 };
            pub const D_RIGHT: Coor<usize> = Coor { i: 0, j: 1 };
            pub const D4: [Coor<usize>; 4] = [D_UP, D_DOWN, D_LEFT, D_RIGHT];
        }
    }
}

