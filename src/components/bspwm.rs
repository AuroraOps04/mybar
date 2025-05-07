use x11::xmu::XmuLookupLatin4;

use super::{Component, Event, Painter};
use crate::bspwm::{Bspwm, DesktopEnum};
use crate::error::MyBarError;
use std::any::Any;
use std::sync::{Arc, Mutex};

pub struct BspwmComponent<'a> {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    painter: &'a Painter<'a>,
    bspwm: Arc<Mutex<Bspwm>>,
}

impl<'a> BspwmComponent<'a> {
    pub fn new(painter: &'a Painter, bspwm: Arc<Mutex<Bspwm>>) -> Self {
        Self {
            x: 10,
            y: 0,
            width: 260,
            height: 40,
            painter,
            bspwm,
        }
    }
}

impl<'a> Component for BspwmComponent<'a> {
    fn draw(&self) -> Result<(), MyBarError> {
        if let Ok(bspwm) = self.bspwm.lock() {
            let w = self.width + 2 * 10;
            self.painter
                .draw_rounded_background(self.x as f64, w as f64, 10f64, "#475164")?;
            let mut x_offset = self.x as f64 + 10f64;
            for monitor in &bspwm.monitors {
                // 绘制显示器名称
                let monitor_color = if monitor.is_active {
                    "#ff3399"
                } else {
                    "#666666"
                };
                self.painter
                    .draw_text(x_offset, 10.0, &monitor.name, monitor_color)?;
                x_offset += self.painter.text_width(&monitor.name)? + 10.0;

                // 绘制桌面
                for desktop in &monitor.desktops {
                    let (icon, color, show) = match desktop.state {
                        DesktopEnum::FOCUSED => ("●", "#ff3399", true),
                        DesktopEnum::OCCUPIED => ("○", "#ffffff", true),
                        DesktopEnum::URGENT => ("!", "#ff0000", true),
                        DesktopEnum::FREE => ("○", "#666666", false),
                    };
                    if !show {
                        continue;
                    }

                    self.painter.draw_text(x_offset, 10.0, icon, color)?;
                    x_offset += self.painter.text_width(icon)? + 5.0;
                }
                x_offset += 10.0; // 显示器之间的间距
            }
        }
        Ok(())
    }

    fn contains_point(&self, x: i16, y: i16) -> bool {
        x >= self.x
            && x <= self.x + self.width as i16
            && y >= self.y
            && y <= self.y + self.height as i16
    }

    fn handle_event(&self, event: &Event) -> Result<(), MyBarError> {
        match event {
            Event::MouseClick { x, y, button } => {
                if let Ok(bspwm) = self.bspwm.lock() {
                    let mut x_offset = 10.0;
                    for monitor in &bspwm.monitors {
                        let monitor_width = self.painter.text_width(&monitor.name)?;
                        if *x >= x_offset as i16 && *x <= (x_offset + monitor_width) as i16 {
                            // TODO: 实现显示器切换
                            break;
                        }
                        x_offset += monitor_width + 10.0;

                        for desktop in &monitor.desktops {
                            let icon = match desktop.state {
                                DesktopEnum::FOCUSED => "●",
                                DesktopEnum::OCCUPIED => "○",
                                DesktopEnum::URGENT => "!",
                                DesktopEnum::FREE => "○",
                            };
                            let icon_width = self.painter.text_width(icon)?;
                            if *x >= x_offset as i16 && *x <= (x_offset + icon_width) as i16 {
                                // TODO: 实现桌面切换
                                break;
                            }
                            x_offset += icon_width + 5.0;
                        }
                        x_offset += 20.0;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn get_bounds(&self) -> (i16, i16, u16, u16) {
        (self.x, self.y, self.width, self.height)
    }
}
