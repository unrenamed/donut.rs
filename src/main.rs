use std::f64::consts::PI;
use std::io::{self, Write};
use std::{thread, time};

use crossterm::{cursor, execute, queue, terminal, Result};

const SCREEN_WIDTH: usize = 30;
const SCREEN_HEIGHT: usize = 30;

const THETA_SPACING: f64 = 0.07;
const PHI_SPACING: f64 = 0.02;

const R1: f64 = 1.0;
const R2: f64 = 2.0;
const K2: f64 = 5.0;

// Calculate K1 based on screen size: the maximum x-distance occurs
// roughly at the edge of the torus, which is at x=R1+R2, z=0.  we
// want that to be displaced 3/8ths of the width of the screen, which
// is 3/4th of the way from the center to the side of the screen.
// SCREEN_WIDTH*3/8 = K1*(R1+R2)/(K2+0)
// SCREEN_WIDTH*K2*3/(8*(R1+R2)) = K1
const K1: f64 = SCREEN_WIDTH as f64 * K2 * 3.0 / (8.0 * (R1 + R2));

struct App<W> {
    output: [[char; SCREEN_WIDTH]; SCREEN_HEIGHT],
    zbuffer: [[f64; SCREEN_WIDTH]; SCREEN_HEIGHT],
    buf: W,
}

impl<W: Write> App<W> {
    pub fn new(buf: W) -> Self {
        Self {
            output: [[' '; SCREEN_WIDTH]; SCREEN_HEIGHT],
            zbuffer: [[0.0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            buf,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.clear_terminal()?;

        let mut a = 1.0;
        let mut b = 1.0;

        loop {
            thread::sleep(time::Duration::from_millis(15));

            self.clear_state();
            self.render_frame(a, b)?;

            a += 0.07;
            b += 0.03;
        }
    }

    fn clear_state(&mut self) {
        self.output = [[' '; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.zbuffer = [[0.0; SCREEN_WIDTH]; SCREEN_HEIGHT];
    }

    fn clear_terminal(&mut self) -> Result<()> {
        execute!(self.buf, terminal::EnterAlternateScreen)?;
        queue!(self.buf, cursor::Hide)?;
        self.buf.flush()?;

        Ok(())
    }

    fn render_frame(&mut self, a: f64, b: f64) -> Result<()> {
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

                // x and y projection. note that y is negated here, because y
                // goes up in 3D space but down on 2D displays.
                let xp = (SCREEN_WIDTH as f64 / 2.0 + K1 * ooz * x) as usize;
                let yp = (SCREEN_HEIGHT as f64 / 2.0 - K1 * ooz * y) as usize;

                // calculate luminance.  ugly, but correct.
                let l =
                    cos_phi * cos_tetha * sin_b - cos_a * cos_tetha * sin_phi - sin_a * sin_tetha
                        + cos_b * (cos_a * sin_tetha - cos_tetha * sin_a * sin_phi);
                // L ranges from -sqrt(2) to +sqrt(2).  If it's < 0, the surface
                // is pointing away from us, so we won't bother trying to plot it.
                if l > 0.0 {
                    // test against the z-buffer.  larger 1/z means the pixel is
                    // closer to the viewer than what's already plotted.
                    if ooz > self.zbuffer[yp][xp] {
                        self.zbuffer[yp][xp] = ooz;

                        // luminance_index is now in the range 0..11 (8*sqrt(2) = 11.3)
                        let luminance_index = l * 8.0;

                        // now we lookup the character corresponding to the luminance and plot it in our output:
                        let ch = String::from(".,-~:;=!*#$@")
                            .chars()
                            .nth(luminance_index as usize)
                            .unwrap();

                        self.output[yp][xp] = ch;
                    }
                }

                phi += PHI_SPACING;
            }

            theta += THETA_SPACING;
        }

        writeln!(self.buf, "\r\x1b[H")?;
        for i in 0..SCREEN_HEIGHT {
            for j in 0..SCREEN_WIDTH {
                write!(self.buf, "{}", self.output[i][j])?;
            }
            writeln!(self.buf)?;
        }

        self.buf.flush()?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();

    let mut app = App::new(&mut lock);
    app.run()?;

    Ok(())
}
