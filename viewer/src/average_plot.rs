use ordered_float::OrderedFloat;
use serde::Serialize;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::Range;
use yew::{function_component, html, Html, Properties};
use yew_plotly::plotly::color::{Rgb, Rgba};
use yew_plotly::plotly::common::{DashType, Fill, Line, Mode, Title};
use yew_plotly::plotly::layout::Axis;
use yew_plotly::plotly::{Layout, Plot, Scatter, Trace};
use yew_plotly::Plotly;

#[derive(Properties, PartialEq)]
pub struct AveragePlotProps<T, const N: usize>
where
    T: Copy + PartialEq,
{
    pub y_data: [Vec<T>; N],
    pub x_name: String,
    pub y_name: String,
    pub main_series_name: String,
    pub main_series_color: (u8, u8, u8),
    pub avg_series_name: String,
    pub avg_series_color: (u8, u8, u8),
}

impl AveragePlotProps<f32, 1> {
    fn y_range(&self) -> Range<f32> {
        let len = self.y_data[0].len();
        if len == 0 {
            return 0.0..0.0;
        }

        let max = self.y_data[0]
            .iter()
            .max_by_key(|&&x| OrderedFloat(x))
            .unwrap();
        let min = self.y_data[0]
            .iter()
            .min_by_key(|&&x| OrderedFloat(x))
            .unwrap();

        let difference = max - min;
        let margin = difference / len as f32 * 10.0;

        (min - margin)..(max + margin)
    }

    fn sliding_average<const N: usize>(&self) -> Vec<f32> {
        self.y_data[0]
            .iter()
            .scan(VecDeque::with_capacity(N), |acc, &next| {
                if acc.len() == N {
                    acc.pop_front();
                }
                acc.push_back(next);
                Some(acc.iter().sum::<f32>() / acc.len() as f32)
            })
            .collect()
    }

    fn avg_series(&self) -> Box<Scatter<usize, f32>> {
        Scatter::new(
            (0..self.y_data[0].len()).into_iter().collect(),
            self.sliding_average::<10>(),
        )
        .mode(Mode::Lines)
        .name(&self.avg_series_name)
        .line(Line::new().color(Rgb::new(
            self.avg_series_color.0,
            self.avg_series_color.1,
            self.avg_series_color.2,
        )))
        .fill(Fill::ToZeroY)
        .fill_color(Rgba::new(
            self.avg_series_color.0,
            self.avg_series_color.1,
            self.avg_series_color.2,
            0.25,
        ))
    }
}

impl<T: Copy + PartialEq + Serialize + 'static + Debug, const N: usize> AveragePlotProps<T, N> {
    fn main_series(&self) -> Vec<Box<dyn Trace>> {
        self.y_data
            .iter()
            .map(|y_data| {
                Scatter::new((0..y_data.len()).into_iter().collect(), y_data.clone())
                    .mode(Mode::Lines)
                    .name(&self.main_series_name)
                    .line(Line::new().dash(DashType::Dot).color(Rgb::new(
                        self.main_series_color.0,
                        self.main_series_color.1,
                        self.main_series_color.2,
                    ))) as Box<dyn Trace>
            })
            .collect()
    }
}

#[function_component]
pub fn AveragePlot<T: Copy + PartialEq + Serialize + 'static + Debug, const N: usize>(
    props: &AveragePlotProps<T, N>,
) -> Html {
    let mut plot = Plot::new();
    plot.set_layout(
        Layout::new()
            .x_axis(Axis::new().title(Title::new(&props.x_name)))
            .y_axis(Axis::new().title(Title::new(&props.y_name)))
            .show_legend(false)
            .auto_size(true),
    );
    plot.add_traces(props.main_series());

    html! { <Plotly {plot}/> }
}

#[function_component]
pub fn AverageF32Plot(props: &AveragePlotProps<f32, 1>) -> Html {
    let y_range = props.y_range();
    let mut plot = Plot::new();
    plot.set_layout(
        Layout::new()
            .x_axis(Axis::new().title(Title::new(&props.x_name)))
            .y_axis(
                Axis::new()
                    .title(Title::new(&props.y_name))
                    .range(vec![y_range.start, y_range.end]),
            )
            .show_legend(false)
            .auto_size(true),
    );
    plot.add_traces(props.main_series());
    plot.add_trace(props.avg_series());

    html! { <Plotly {plot}/> }
}

#[cfg(test)]
mod tests {
    use crate::average_plot::AveragePlotProps;
    use float_cmp::ApproxEq;
    use std::ops::Range;

    #[repr(transparent)]
    #[derive(PartialEq, Debug)]
    struct RangeF32(Range<f32>);

    impl<M: Copy + Default> ApproxEq for RangeF32
    where
        f32: ApproxEq<Margin = M>,
    {
        type Margin = M;

        fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
            let margin = margin.into();
            self.0.start.approx_eq(other.0.start, margin)
                && self.0.end.approx_eq(other.0.end, margin)
        }
    }

    fn default_plot() -> AveragePlotProps<f32, 1> {
        AveragePlotProps {
            y_data: [vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]],
            x_name: "".to_string(),
            y_name: "".to_string(),
            main_series_name: "Test".to_string(),
            main_series_color: (0, 0, 0),
            avg_series_name: "".to_string(),
            avg_series_color: (0, 0, 0),
        }
    }

    #[test]
    fn test_avg_plot_range() {
        let props = default_plot();

        assert_eq!(RangeF32(props.y_range()), RangeF32(-8.0..19.0))
    }

    #[test]
    fn test_avg_plot_cum_avg() {
        let props = default_plot();

        assert_eq!(
            props.sliding_average::<4>(),
            vec![1.0, 1.5, 2.0, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5]
        );
    }
}
