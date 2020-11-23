use color_eyre::Help;
use iced::image::Handle as ImageHandle;
use plotters::{
    prelude::{ChartBuilder, IntoDrawingArea, LineSeries, PointSeries, SVGBackend},
    style::{Color, IntoFont, RGBAColor, ShapeStyle, RED, WHITE},
};
use plotters_bitmap::{bitmap_pixel::BGRXPixel, BitMapBackend};

use crate::simulation::motor::RocketMotor;

const GRAPH_WIDTH: u32 = 1024;
const GRAPH_HEIGHT: u32 = 1024;
const BUFFER_SIZE: usize = (GRAPH_HEIGHT * GRAPH_WIDTH * 4) as usize;

type GraphBuf = [u8; BUFFER_SIZE];

#[derive(Debug)]
pub struct GraphBuffer(Box<GraphBuf>);

impl GraphBuffer {
    pub fn new() -> Self {
        Self(Box::new([0; BUFFER_SIZE]))
    }

    pub fn as_handle(&self) -> ImageHandle {
        ImageHandle::from_pixels(GRAPH_WIDTH, GRAPH_HEIGHT, self.0.to_vec())
    }

    pub fn pixels(&self) -> &GraphBuf {
        &self.0
    }

    pub fn pixels_mut(&mut self) -> &mut GraphBuf {
        &mut self.0
    }
}

pub fn draw_motor_graph(buf: &mut String, motor: RocketMotor) -> color_eyre::Result<()> {
    let plot = SVGBackend::with_string(buf, (GRAPH_WIDTH, GRAPH_HEIGHT)).into_drawing_area();

    plot.fill(&WHITE).note("Failed to draw the background")?;

    let datapoints = (motor.min.floor() as i64..=(motor.max * 100.0).ceil() as i64)
        .map(|i| i as f64 * 0.01)
        .map(|i| (i, (motor.thrust)(i).thrust))
        .collect::<Vec<_>>();

    // After this point, we should be able to draw construct a chart context
    let mut chart = ChartBuilder::on(&plot)
        // Set the caption of the chart
        .caption("This is our first plot", ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(
            motor.min..motor.max,
            0f64..datapoints
                .iter()
                .map(|(_x, y)| y.ceil() as i64)
                .max()
                .unwrap_or_default() as f64,
        )
        .unwrap();

    // Then we can draw a mesh
    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        .label_style(("sans-serif", 80).into_font())
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            datapoints,
            ShapeStyle {
                color: RED.to_rgba(),
                filled: false,
                stroke_width: 20,
            },
        ))
        .unwrap()
        .label("H");

    Ok(())
}
