use ordered_float::OrderedFloat;
use std::ops::Range;
use yew::{function_component, html, Html, Properties};
use yew_plotly::plotly::color::Rgb;
use yew_plotly::plotly::common::{DashType, Fill, Line, Mode, Title};
use yew_plotly::plotly::layout::Axis;
use yew_plotly::plotly::{Layout, Plot, Scatter};
use yew_plotly::Plotly;

#[derive(Properties, PartialEq)]
pub struct AveragePlotProps {
    pub y_data: Vec<f32>,
    pub x_name: String,
    pub y_name: String,
    pub main_series_name: String,
    pub main_series_color: (u8, u8, u8),
    pub avg_series_name: String,
    pub avg_series_color: (u8, u8, u8),
}

impl AveragePlotProps {
    fn y_range(&self) -> Range<f32> {
        let len = self.y_data.len();
        if len == 0 {
            return 0.0..0.0;
        }

        let max = self
            .y_data
            .iter()
            .max_by_key(|&&x| OrderedFloat(x))
            .unwrap();
        let min = self
            .y_data
            .iter()
            .min_by_key(|&&x| OrderedFloat(x))
            .unwrap();

        let difference = max - min;
        let margin = difference / len as f32 * 10.0;

        (min - margin)..(max + margin)
    }

    fn cumulative_average(&self) -> Vec<f32> {
        self.y_data
            .iter()
            .scan((0.0, 0), |(acc, cnt), x| {
                *cnt += 1;
                *acc += *x;
                Some(*acc / *cnt as f32)
            })
            .collect()
    }

    fn main_series(&self) -> Box<Scatter<usize, f32>> {
        Scatter::new(
            (0..self.y_data.len()).into_iter().collect(),
            self.y_data.clone(),
        )
        .mode(Mode::Lines)
        .name(&self.main_series_name)
        .line(Line::new().dash(DashType::Dot).color(Rgb::new(
            self.main_series_color.0,
            self.main_series_color.1,
            self.main_series_color.2,
        )))
    }

    fn avg_series(&self) -> Box<Scatter<usize, f32>> {
        let tint_factor = 3. / 4.;
        let tint = (
            self.avg_series_color.0 as f64 + (255 - self.avg_series_color.0) as f64 * tint_factor,
            self.avg_series_color.1 as f64 + (255 - self.avg_series_color.1) as f64 * tint_factor,
            self.avg_series_color.2 as f64 + (255 - self.avg_series_color.2) as f64 * tint_factor,
        );

        Scatter::new(
            (0..self.y_data.len()).into_iter().collect(),
            self.cumulative_average(),
        )
        .mode(Mode::Lines)
        .name(&self.avg_series_name)
        .line(Line::new().color(Rgb::new(
            self.avg_series_color.0,
            self.avg_series_color.1,
            self.avg_series_color.2,
        )))
        .fill(Fill::ToZeroY)
        .fill_color(Rgb::new(tint.0 as u8, tint.1 as u8, tint.2 as u8))
    }

    fn layout(&self) -> Layout {
        let range = self.y_range();

        Layout::new()
            .x_axis(Axis::new().title(Title::new(&self.x_name)))
            .y_axis(
                Axis::new()
                    .title(Title::new(&self.y_name))
                    .range(vec![range.start, range.end]),
            )
            .show_legend(false)
            .auto_size(true)
    }
}

#[function_component]
pub fn AveragePlot(props: &AveragePlotProps) -> Html {
    let mut plot = Plot::new();
    plot.set_layout(props.layout());
    plot.add_traces(vec![props.avg_series(), props.main_series()]);

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

    fn default_plot() -> AveragePlotProps {
        AveragePlotProps {
            y_data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
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
            props.cumulative_average(),
            vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5]
        );
    }
}
