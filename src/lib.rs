use chrono::{Date, Datelike, Local, Weekday};

/// Gets the date that represents the upcoming weekday. Given tomorrow’s weekday, this should return
/// tomorrows date. Today is represented by the current weekday.
fn weekday_to_date(weekday: Weekday, mut today: Date<Local>) -> Date<Local> {
    loop {
        if today.weekday() == weekday {
            return today;
        }
        today = today.succ();
    }
}

/// Represents a one-time task to be done at a specific date. The date is specified as a weekday
/// from now. If no weekday is given, the current weekday will be used. After the given weekday, the
/// `Todo` will show up for the current day.
pub struct Todo {
    body: String,
    date: Date<Local>,
}

impl Todo {
    /// Creates a new `Todo` that shows up to be done for the current day.
    pub fn new_undated(body: String) -> Todo {
        Todo {
            body,
            date: Local::today(),
        }
    }

    /// Creates a new `Todo` that shows up to be done at a specific weekday after which it will show
    /// for the current day.
    pub fn new_dated(body: String, weekday: Weekday) -> Todo {
        Todo {
            body,
            date: weekday_to_date(weekday, Local::today()),
        }
    }

    /// Returns `true` if the `Todo` is for a given weekday.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Datelike, Local, Weekday};
    /// use mtd::Todo;
    ///
    /// let td = Todo::new_undated("For today's weekday.".to_string());
    /// assert!(td.for_weekday(Local::today().weekday()));
    ///
    /// let td = Todo::new_dated("For the next wednesday.".to_string(), Weekday::Wed);
    /// assert!(td.for_weekday(Weekday::Wed));
    ///
    /// let td = Todo::new_dated("For the next wednesday.".to_string(), Weekday::Wed);
    /// assert!(!td.for_weekday(Weekday::Tue));
    /// ```
    pub fn for_weekday(&self, weekday: Weekday) -> bool {
        self.weekday() == weekday
    }

    /// Gets the `body` of the `Todo`.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Gets the weekday of the `Todo`.
    pub fn weekday(&self) -> Weekday {
        self.date.weekday()
    }

    /// Sets the `body` of the `Todo`.
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Sets the weekday of the `Todo`.
    pub fn set_weekday(&mut self, weekday: Weekday) {
        self.date = weekday_to_date(weekday, Local::today());
    }
}

/// Represents a reoccurring task for the given weekday(s).
pub struct Task {
    body: String,
    weekdays: Vec<Weekday>,
}

impl Task {
    /// Creates a new task with the given weekday(s).
    ///
    /// # Panics
    ///
    /// If the given weekdays list is empty.
    pub fn new(body: String, weekdays: Vec<Weekday>) -> Task {
        if weekdays.is_empty() {
            panic!("Cannot create a task without specifying at least one weekday.")
        }
        Task { body, weekdays }
    }

    /// Gets the `body` of the `Task`.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Gets the `weekdays` of the `Task`.
    pub fn weekdays(&self) -> &Vec<Weekday> {
        &self.weekdays
    }

    /// Sets the `body` of the `Task`.
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Sets the `weekdays` of the `Task`.
    pub fn set_weekdays(&mut self, weekdays: Vec<Weekday>) {
        self.weekdays = weekdays;
    }

    /// Adds a weekday to the weekdays list.
    pub fn add_weekday(&mut self, weekday: Weekday) {
        // It doesn't matter if there are duplicate weekdays.
        self.weekdays.push(weekday);
    }

    /// Removes a weekday from the weekdays list.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::Weekday;
    /// use mtd::Task;
    ///
    /// let mut task = Task::new("Test task".to_string(), vec![Weekday::Mon, Weekday::Tue, Weekday::Wed]);
    /// task.remove_weekday(Weekday::Wed);
    ///
    /// // Removing a weekday that isn't listed does nothing.
    /// task.remove_weekday(Weekday::Fri);
    ///
    /// assert!(task.weekdays().contains(&Weekday::Mon));
    /// assert!(task.weekdays().contains(&Weekday::Tue));
    /// // Doesn't contain wed anymore
    /// assert!(!task.weekdays().contains(&Weekday::Wed));
    /// ```
    pub fn remove_weekday(&mut self, removed_wd: Weekday) {
        let mut new_weekdays = Vec::new();

        for wd in &self.weekdays {
            if wd != &removed_wd {
                new_weekdays.push(wd.clone());
            }
        }

        self.weekdays = new_weekdays;
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone, Weekday};

    use crate::{Task, weekday_to_date};

    // Unit test a private function to remove the need to pass today into the Todo constructor
    #[test]
    fn weekday_to_date_returns_correct_dates() {
        // Today is a Tuesday
        let today = Local.ymd(2022, 6, 7);

        // Tue should return today’s date
        assert_eq!(weekday_to_date(Weekday::Tue, today), today);

        // Wed should return tomorrow’s date
        assert_eq!(weekday_to_date(Weekday::Wed, today), today.succ());

        // Mon should return next weeks monday
        assert_eq!(weekday_to_date(Weekday::Mon, today), Local.ymd(2022, 6, 13));
    }

    #[test]
    #[should_panic]
    fn task_new_panics_if_empty_weekday_vec() {
        let task = Task::new("Panic!".to_string(), vec![]);
    }

    #[test]
    fn task_remove_weekday_removes_all_duplicates() {
        let mut task = Task::new("Test task".to_string(), vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Wed]);

        task.remove_weekday(Weekday::Wed);

        assert!(task.weekdays().contains(&Weekday::Mon));
        assert!(task.weekdays().contains(&Weekday::Tue));
        assert!(!task.weekdays().contains(&Weekday::Wed));
    }
}