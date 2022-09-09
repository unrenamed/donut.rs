use std::f64::consts::PI;
use std::io::{self, Write};
use std::{thread, time};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

static THETA_SPACING: f64 = 0.3;
static PHI_SPACING: f64 = 0.1;

static R1: f64 = 1.2;
static R2: f64 = 2.0;
static K1: f64 = 30.0;
static K2: f64 = 6.5;

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    queue!(
        w,
        style::ResetColor,
        terminal::Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(1, 1)
    )?;

    w.flush()?;

    let mut a = 1.0;
    let mut b = 1.0;
    loop {
        render_frame(w, a, b);
        a += 0.07;
        b += 0.03;

        queue!(w, terminal::Clear(ClearType::All))?;
        w.flush()?;
        thread::sleep(time::Duration::from_millis(10));
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    terminal::disable_raw_mode()
}

fn render_frame<W>(w: &mut W, a: f64, b: f64) -> Result<()>
where
    W: Write,
{
    // precompute sines and cosines of A and B
    let cos_a = a.cos();
    let sin_a = a.sin();
    let cos_b = b.cos();
    let sin_b = b.sin();

    // theta goes around the cross-sectional circle of a torus
    let mut theta: f64 = 0.0;
    while theta < 2.0 * PI {
        // precompute sines and cosines of theta
        let cos_tetha = theta.cos();
        let sin_tetha = theta.sin();

        // phi goes around the center of revolution of a torus
        let mut phi: f64 = 0.0;
        while phi < 2.0 * PI {
            // precompute sines and cosines of phi
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();

            // the x,y coordinate of the circle, before revolving (factored
            // out of the above equations)
            let cx = R2 + R1 * cos_tetha;
            let cy = R1 * sin_tetha;

            // final 3D (x,y,z) coordinate after rotations, directly from
            // our math above
            let x = cx * (cos_b * cos_phi + sin_a * sin_b * sin_phi) - cy * cos_a * sin_b;
            let y = cx * (sin_b * cos_phi - sin_a * cos_b * sin_phi) + cy * cos_a * cos_b;
            let z = K2 + cos_a * cx * sin_phi + cy * sin_a;
            let ooz = 1.0 / z; // "one over z"

            // x and y projection.  note that y is negated here, because y
            // goes up in 3D space but down on 2D displays.
            let (width, height) = terminal::size().unwrap();
            let xp = width as f64 / 2.0 + K1 * ooz * x;
            let yp = height as f64 / 2.0 - K1 * ooz * y;

            // calculate luminance.  ugly, but correct.
            let L = cos_phi * cos_tetha * sin_b - cos_a * cos_tetha * sin_phi - sin_a * sin_tetha
                + cos_b * (cos_a * sin_tetha - cos_tetha * sin_a * sin_phi);
            // L ranges from -sqrt(2) to +sqrt(2).  If it's < 0, the surface
            // is pointing away from us, so we won't bother trying to plot it.
            if L > 0.0 {
                let luminance_index = L * 8.0;
                let ch = String::from(".,-~:;=!*#$@")
                    .chars()
                    .nth(luminance_index as usize)
                    .unwrap();

                queue!(w, cursor::MoveTo(xp as u16, yp as u16), style::Print(ch))?;
            }

            phi += PHI_SPACING;
        }

        theta += THETA_SPACING;
    }

    w.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = io::stdout();
    run(&mut stdout)
}
