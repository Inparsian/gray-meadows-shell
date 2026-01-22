use std::rc::Rc;
use std::cell::RefCell;
use gtk4::prelude::*;
use futures_signals::signal::SignalExt as _;
use chrono::Datelike as _;
use relm4::RelmRemoveAllExt as _;
use num_traits::cast::FromPrimitive as _;

use crate::singletons::date_time::DATE_TIME;

pub enum CalendarMarker {
    OutsideMonth,
    InsideMonth,
    Today
}

pub struct CalendarDay {
    pub day: u32,
    pub marker: CalendarMarker,
}

pub struct CalendarWeek {
    pub days: Vec<CalendarDay>,
}

#[derive(Default, Clone, glib::Downgrade)]
pub struct Calendar {
    pub weeks: Rc<RefCell<Vec<CalendarWeek>>>,
    pub month: Rc<RefCell<u32>>,
    pub year: Rc<RefCell<i32>>,
    pub root: Rc<RefCell<Option<gtk4::Box>>>,
    pub current_date_label: Rc<RefCell<Option<gtk4::Label>>>,
    pub days_grid: Rc<RefCell<Option<gtk4::Box>>>,
}

impl Calendar {
    pub fn new() -> Self {
        let now = chrono::Local::now();
        let month = now.month();
        let year = now.year();
        let weeks = Vec::new();

        Calendar {
            weeks: Rc::new(RefCell::new(weeks)),
            month: Rc::new(RefCell::new(month)),
            year: Rc::new(RefCell::new(year)),
            ..Default::default()
        }
    }

    pub fn rebuild_weeks(&self) {
        let now = chrono::Local::now();
        let day = now.day();
        let month = *self.month.borrow();
        let year = *self.year.borrow();
        let weekday_of_month_first = {
            let first_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            first_of_month.weekday().num_days_from_monday()
        };
        let days_in_month = get_month_days(month, year);
        let days_in_prev_month = if month == 1 {
            get_month_days(12, year - 1)
        } else {
            get_month_days(month - 1, year)
        };

        let mut weeks: Vec<CalendarWeek> = Vec::new();
        let mut current_week = CalendarWeek {
            days: Vec::new()
        };

        while weeks.len() < 6 {
            while current_week.days.len() < 7 {
                let current_day_index = weeks.len() * 7 + current_week.days.len();
                let mut calendar_day = CalendarDay {
                    day: 0,
                    marker: CalendarMarker::OutsideMonth,
                };

                if current_day_index < weekday_of_month_first as usize {
                    calendar_day.day = days_in_prev_month - ((weekday_of_month_first as usize - current_day_index) - 1) as u32;
                } else if current_day_index >= (weekday_of_month_first as usize + days_in_month as usize) {
                    calendar_day.day = (current_day_index - (weekday_of_month_first as usize + days_in_month as usize) + 1) as u32;
                } else {
                    let calc_day = (current_day_index - weekday_of_month_first as usize + 1) as u32;

                    calendar_day.day = calc_day;
                    calendar_day.marker = if calc_day == day {
                        CalendarMarker::Today
                    } else {
                        CalendarMarker::InsideMonth
                    }
                }

                current_week.days.push(calendar_day);
            }

            weeks.push(current_week);
            current_week = CalendarWeek {
                days: Vec::new()
            };
        }

        *self.weeks.borrow_mut() = weeks;
    }

    pub fn make_widget(&self) -> gtk4::Box {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_css_classes(&["calendar-tab-calendar"]);

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header.set_css_classes(&["calendar-header"]);
        root.append(&header);

        let current_date_label = gtk4::Label::new(None);
        current_date_label.set_css_classes(&["calendar-header-current-date"]);
        current_date_label.set_xalign(0.0);
        current_date_label.set_hexpand(true);
        current_date_label.set_halign(gtk4::Align::Start);
        header.append(&current_date_label);

        let prev_year_button = gtk4::Button::new();
        prev_year_button.set_css_classes(&["calendar-header-button"]);
        prev_year_button.set_label("keyboard_double_arrow_left");
        prev_year_button.connect_clicked(clone!(
            #[weak(rename_to = me)] self,
            move |_| {
                me.shift_date_x_months(-12);
                me.render();
            }
        ));
        header.append(&prev_year_button);

        let prev_month_button = gtk4::Button::new();
        prev_month_button.set_css_classes(&["calendar-header-button"]);
        prev_month_button.set_label("chevron_left");
        prev_month_button.connect_clicked(clone!(
            #[weak(rename_to = me)] self,
            move |_| {
                me.shift_date_x_months(-1);
                me.render();
            }
        ));
        header.append(&prev_month_button);

        let today_button = gtk4::Button::new();
        today_button.set_css_classes(&["calendar-header-button"]);
        today_button.set_label("today");
        today_button.connect_clicked(clone!(
            #[weak(rename_to = me)] self,
            move |_| {
                me.set_to_today();
                me.render();
            }
        ));
        header.append(&today_button);

        let next_month_button = gtk4::Button::new();
        next_month_button.set_css_classes(&["calendar-header-button"]);
        next_month_button.set_label("chevron_right");
        next_month_button.connect_clicked(clone!(
            #[weak(rename_to = me)] self,
            move |_| {
                me.shift_date_x_months(1);
                me.render();
            }
        ));
        header.append(&next_month_button);

        let next_year_button = gtk4::Button::new();
        next_year_button.set_css_classes(&["calendar-header-button"]);
        next_year_button.set_label("keyboard_double_arrow_right");
        next_year_button.connect_clicked(clone!(
            #[weak(rename_to = me)] self,
            move |_| {
                me.shift_date_x_months(12);
                me.render();
            }
        ));
        header.append(&next_year_button);

        let calendar_body = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        calendar_body.set_css_classes(&["calendar-body"]);
        root.append(&calendar_body);

        let weekdays_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        weekdays_box.set_css_classes(&["calendar-body-weekdays-box"]);
        weekdays_box.set_homogeneous(true);
        calendar_body.append(&weekdays_box);

        for weekday_name in &["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] {
            let weekday_label = gtk4::Label::new(Some(weekday_name));
            weekday_label.set_css_classes(&["calendar-body-weekday-label"]);
            weekdays_box.append(&weekday_label);
        }

        let days_grid = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        days_grid.set_css_classes(&["calendar-body-days-grid"]);
        calendar_body.append(&days_grid);

        self.root.replace(Some(root.clone()));
        self.current_date_label.replace(Some(current_date_label));
        self.days_grid.replace(Some(days_grid));
        self.rebuild_weeks();
        self.render();
        root
    }

    pub fn render(&self) {
        if let Some(current_date_label) = self.current_date_label.borrow().as_ref() {
            let month = *self.month.borrow();
            let year = *self.year.borrow();
            let month_name = chrono::Month::from_u32(month).unwrap().name();
            current_date_label.set_text(&format!("{} {}", month_name, year));
        }

        if let Some(days_grid) = self.days_grid.borrow().as_ref() {
            days_grid.remove_all();

            for week in self.weeks.borrow().iter() {
                let week_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
                week_box.set_css_classes(&["calendar-grid-week-box"]);
                week_box.set_homogeneous(true);
                days_grid.append(&week_box);

                for day in &week.days {
                    let day_label = gtk4::Label::new(Some(&day.day.to_string()));
                    day_label.set_css_classes(&["calendar-grid-day-label"]);

                    match day.marker {
                        CalendarMarker::OutsideMonth => {
                            day_label.add_css_class("outside-month");
                        },

                        CalendarMarker::InsideMonth => {
                            day_label.add_css_class("inside-month");
                        },

                        CalendarMarker::Today => {
                            day_label.add_css_class("today");
                        },
                    }

                    week_box.append(&day_label);
                }
            }
        }
    }

    pub fn shift_date_x_months(&self, months: i32) {
        {
            let mut year = self.year.borrow_mut();
            let mut month = self.month.borrow_mut();
            let total_months = *year * 12 + (*month as i32 - 1) + months;
            *year = total_months / 12;
            *month = (total_months % 12 + 1) as u32;
        }

        self.rebuild_weeks();
    }

    pub fn set_to_today(&self) {
        let now = chrono::Local::now();
        *self.month.borrow_mut() = now.month();
        *self.year.borrow_mut() = now.year();
        self.rebuild_weeks();
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 400 == 0) || (year % 4 == 0 && year % 100 != 0)
}

fn get_month_days(month: u32, year: i32) -> u32 {
    let is_leap = is_leap_year(year);

    if (month <= 7 && month % 2 == 1) || (month >= 8 && month.is_multiple_of(2)) {
        31
    }

    else if month == 2 && is_leap {
        29
    }

    else if month == 2 && !is_leap {
        28
    }

    else {
        30
    }
}

pub fn new() -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.set_css_classes(&["calendar-tab-root"]);

    let calendar = Calendar::new();
    let calendar_widget = calendar.make_widget();

    root.append(&calendar_widget);

    // When the date changes, re-render the calendar
    let current_date = Rc::new(RefCell::new(DATE_TIME.get_cloned().date));
    glib::spawn_future_local(signal_cloned!(DATE_TIME, (date_time) {
        let mut lock = current_date.borrow_mut();
        if date_time.date != *lock {
            *lock = date_time.date;
            calendar.render();
        }
    }));

    root
}