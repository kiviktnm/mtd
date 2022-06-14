use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use chrono::{Date, Datelike, Local, Weekday};

// Methods ending with _wtd are used for unit testing and internal implementations. They allow
// supplying today with any date.

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
#[derive(Debug, PartialEq)]
pub struct Todo {
    body: String,
    date: Date<Local>,
    id: u64,
    done: Option<Date<Local>>,
}

impl Todo {
    /// Creates a new `Todo` that shows up to be done for the current day.
    pub fn new_undated(body: String) -> Todo {
        Todo {
            body,
            date: Local::today(),
            id: 0,
            done: None,
        }
    }

    /// Creates a new `Todo` that shows up to be done at a specific weekday after which it will show
    /// for the current day.
    pub fn new_dated(body: String, weekday: Weekday) -> Todo {
        Todo {
            body,
            date: weekday_to_date(weekday, Local::today()),
            id: 0,
            done: None,
        }
    }

    // Used for unit testing with non-today dependant date
    #[cfg(test)]
    fn new_specific_date(body: String, date: Date<Local>) -> Todo {
        Todo {
            body,
            date,
            id: 0,
            done: None,
        }
    }

    /// Returns `true` if the `Todo` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Datelike, Local};
    /// use mtd::Todo;
    ///
    /// let todo_for_today = Todo::new_undated("I am for today".to_string());
    ///
    /// assert!(todo_for_today.for_date(Local::today()));
    ///
    /// let todo_for_tomorrow = Todo::new_dated("I am for tomorrow".to_string(), Local::today().succ().weekday());
    ///
    /// assert!(!todo_for_tomorrow.for_date(Local::today()));
    /// assert!(todo_for_tomorrow.for_date(Local::today().succ()));
    /// ```
    pub fn for_date(&self, date: Date<Local>) -> bool {
        self.for_date_wtd(date, Local::today())
    }

    fn for_date_wtd(&self, date: Date<Local>, today: Date<Local>) -> bool {
        date >= self.date && (date == today || self.date > today)
    }

    /// Gets the `body` of the `Todo`.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Gets the weekday of the `Todo`.
    pub fn weekday(&self) -> Weekday {
        self.date.weekday()
    }

    /// Gets the `id` of the `Todo`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Sets the `body` of the `Todo`.
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Sets the weekday of the `Todo`.
    pub fn set_weekday(&mut self, weekday: Weekday) {
        self.date = weekday_to_date(weekday, Local::today());
    }

    /// Sets the `id` of the `Todo`.
    fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    /// Returns `true` if the `Todo` is done.
    pub fn done(&self) -> bool {
        self.done.is_some()
    }

    /// Sets the done state of the `Todo`.
    pub fn set_done(&mut self, done: bool) {
        self.set_done_wtd(done, Local::today());
    }

    fn set_done_wtd(&mut self, done: bool, today: Date<Local>) {
        if done {
            self.done = Some(today);
        } else {
            self.done = None;
        }
    }

    /// Returns `true` if the `Todo` can be removed. A `Todo` can be removed one day after its
    /// completion.
    pub fn can_remove(&self) -> bool {
        self.can_remove_wtd(Local::today())
    }

    fn can_remove_wtd(&self, today: Date<Local>) -> bool {
        if let Some(done_date) = self.done {
            today > done_date
        } else {
            false
        }
    }
}

impl Display for Todo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.body, self.id)
    }
}

/// Represents a reoccurring task for the given weekday(s).
#[derive(Debug, PartialEq)]
pub struct Task {
    body: String,
    weekdays: Vec<Weekday>,
    done_map: HashMap<Weekday, Date<Local>>,
    id: u64,
}

impl Task {
    /// Creates a new task for the given weekday(s).
    ///
    /// # Panics
    ///
    /// If the given weekdays list is empty.
    pub fn new(body: String, weekdays: Vec<Weekday>) -> Task {
        if weekdays.is_empty() {
            panic!("Cannot create a task without specifying at least one weekday.")
        }
        Task { body, weekdays, id: 0, done_map: HashMap::new() }
    }

    /// Gets the `body` of the `Task`.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Gets the `weekdays` of the `Task`. Note that duplicate weekdays are allowed.
    pub fn weekdays(&self) -> &Vec<Weekday> {
        &self.weekdays
    }

    /// Gets the `id` of the `Task`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Sets the `body` of the `Task`.
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Sets the `weekdays` of the `Task`.
    pub fn set_weekdays(&mut self, weekdays: Vec<Weekday>) {
        self.weekdays = weekdays;
    }

    /// Sets the `id` of the `Task`.
    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    /// Adds a weekday to the weekdays list.
    pub fn add_weekday(&mut self, weekday: Weekday) {
        // It doesn't matter if there are duplicate weekdays.
        self.weekdays.push(weekday);
    }

    /// Removes a weekday from the weekdays list. Removes all duplicates as well.
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

        self.set_weekdays(new_weekdays);
    }

    /// Returns `true` if the `Task` is for a given date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Local, TimeZone, Weekday};
    /// use mtd::Task;
    ///
    /// let task = Task::new("Task".to_string(), vec![Weekday::Fri, Weekday::Sun]);
    ///
    /// assert!(task.for_date(Local.ymd(2022, 6, 10))); // 2022-6-10 is a Friday
    /// assert!(!task.for_date(Local.ymd(2022, 6, 11))); // Saturday
    /// assert!(task.for_date(Local.ymd(2022, 6, 12))); // Sunday
    /// ```
    pub fn for_date(&self, date: Date<Local>) -> bool {
        self.weekdays.contains(&date.weekday())
    }

    /// Returns `true` if the `Task` is done for the given date. Always returns `true` if the task
    /// is not for the given the date.
    ///
    /// # Example
    ///
    /// ```
    /// use chrono::{Local, TimeZone, Weekday};
    /// use mtd::Task;
    ///
    /// let mut task = Task::new("Task".to_string(), vec![Weekday::Mon, Weekday::Wed, Weekday::Thu]);
    ///
    /// task.set_done(true, Local.ymd(2022, 6, 13));
    /// task.set_done(true, Local.ymd(2022, 6, 16));
    ///
    /// // Done for mon and thu
    /// assert!(task.done(Local.ymd(2022, 6, 13)));
    /// assert!(task.done(Local.ymd(2022, 6, 16)));
    ///
    /// // Not done for wed
    /// assert!(!task.done(Local.ymd(2022, 6, 15)));
    ///
    /// // Not done for the following week's mon/thu
    /// assert!(!task.done(Local.ymd(2022, 6, 20)));
    /// assert!(!task.done(Local.ymd(2022, 6, 23)));
    ///
    /// // Since 2022-6-21 is a tue, the task is done for that date
    /// assert!(task.done(Local.ymd(2022, 6, 21)));
    /// ```
    pub fn done(&self, date: Date<Local>) -> bool {
        if self.for_date(date) {
            if let Some(d) = self.done_map.get(&date.weekday()) {
                return *d >= date;
            }
            return false;
        }
        true
    }


    /// Sets the done state of the `Task` for the given date.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// use chrono::{Local, TimeZone, Weekday};
    /// use mtd::Task;
    ///
    /// let mut task = Task::new("Task".to_string(), vec![Weekday::Mon]);
    ///
    /// task.set_done(true, Local.ymd(2022, 6, 13));
    /// assert!(task.done(Local.ymd(2022, 6, 13)));
    ///
    /// task.set_done(false, Local.ymd(2022, 6, 13));
    /// assert!(!task.done(Local.ymd(2022, 6, 13)));
    /// ```
    pub fn set_done(&mut self, done: bool, date: Date<Local>) {
        if done {
            self.done_map.insert(date.weekday(), date);
        } else {
            self.done_map.remove(&date.weekday());
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.body, self.id)
    }
}

/// A synchronizable list used for containing and managing all `Todo`s and `Task`s. `Todo`s and
/// `Task`s have `id`s that match their index within the internal `Vec`s of the `TdList`.
#[derive(Debug)]
pub struct TdList {
    todos: Vec<Todo>,
    tasks: Vec<Task>,
}

/* TdList contains some amount of code duplication, but Tasks and Todos are imo too different
 * to abstract to a common trait with a templated TdList<trait>. Therefore the current amount of
 * duplication will be accepted.
 */

impl TdList {
    /// Creates a new empty `TdList`.
    pub fn new() -> Self {
        Self { todos: Vec::new(), tasks: Vec::new() }
    }

    /// Gets all the `Todo`s in the list. A `Todo`'s id matches its index in this list.
    pub fn todos(&self) -> Vec<&Todo> {
        let mut todos = Vec::new();
        for todo in &self.todos {
            todos.push(todo);
        }

        todos
    }

    /// Gets all the `Task`s in the list. A `Task`'s id matches its index in this list.
    pub fn tasks(&self) -> Vec<&Task> {
        let mut tasks = Vec::new();
        for task in &self.tasks {
            tasks.push(task);
        }

        tasks
    }

    /// Adds a `Todo` to the list and modifies its id.
    pub fn add_todo(&mut self, mut todo: Todo) {
        todo.set_id(self.todos.len() as u64);
        self.todos.push(todo);
    }

    /// Adds a `Task` to the list and modifies its id.
    pub fn add_task(&mut self, mut task: Task) {
        task.set_id(self.tasks.len() as u64);
        self.tasks.push(task);
    }

    /// Removes the `Todo` that matches the given id. The id matches the index of the `Todo`. If no
    /// such `Todo` exists, does nothing. Removing a `Todo` may change the ids of other `Todo`s.
    pub fn remove_todo(&mut self, id: u64) {
        if self.todos.len() <= id as usize {
            return;
        }
        self.todos.remove(id as usize);

        for (new_id, item) in self.todos.iter_mut().enumerate() {
            item.set_id(new_id as u64);
        }
    }

    /// Removes the `Task` that matches the given id. The id matches the index of the `Task`. If no
    /// such `Task` exists, does nothing. Removing a `Task` may change the ids of other `Task`s.
    pub fn remove_task(&mut self, id: u64) {
        if self.tasks.len() <= id as usize {
            return;
        }
        self.tasks.remove(id as usize);

        for (new_id, item) in self.tasks.iter_mut().enumerate() {
            item.set_id(new_id as u64);
        }
    }

    /// Returns a mutable reference to a `Todo`.
    pub fn get_todo_mut(&mut self, id: u64) -> Option<&mut Todo> {
        self.todos.get_mut(id as usize)
    }

    /// Returns a mutable reference to a `Task`.
    pub fn get_task_mut(&mut self, id: u64) -> Option<&mut Task> {
        self.tasks.get_mut(id as usize)
    }

    /// Returns all `Todo`s for a given date that are not yet done.
    pub fn undone_todos_for_date(&self, date: Date<Local>) -> Vec<&Todo> {
        self.undone_todos_for_date_wtd(date, Local::today())
    }

    /// Returns all `Todo`s for a given date that are done.
    pub fn done_todos_for_date(&self, date: Date<Local>) -> Vec<&Todo> {
        self.done_todos_for_date_wtd(date, Local::today())
    }

    fn undone_todos_for_date_wtd(&self, date: Date<Local>, today: Date<Local>) -> Vec<&Todo> {
        let mut undone_todos = Vec::new();

        for todo in &self.todos {
            if todo.for_date_wtd(date, today) && !todo.done() {
                undone_todos.push(todo);
            }
        }

        undone_todos
    }

    fn done_todos_for_date_wtd(&self, date: Date<Local>, today: Date<Local>) -> Vec<&Todo> {
        let mut done_todos = Vec::new();

        for todo in &self.todos {
            if todo.for_date_wtd(date, today) && todo.done() {
                done_todos.push(todo);
            }
        }

        done_todos
    }

    /// Returns all `Task`s for a given date that are not yet done.
    pub fn undone_tasks_for_date(&self, date: Date<Local>) -> Vec<&Task> {
        let mut undone_tasks = Vec::new();

        for task in &self.tasks {
            if task.for_date(date) && !task.done(date) {
                undone_tasks.push(task);
            }
        }

        undone_tasks
    }

    /// Returns all `Task`s for a given date that are done.
    pub fn done_tasks_for_date(&self, date: Date<Local>) -> Vec<&Task> {
        let mut done_tasks = Vec::new();

        for task in &self.tasks {
            if task.for_date(date) && task.done(date) {
                done_tasks.push(task);
            }
        }

        done_tasks
    }

    /// Removes all `Todo`s that are done and at least a day has passed since their completion.
    /// Basically remove all `Todo`s which `Todo.can_remove()` returns `true`.
    pub fn remove_old_todos(&mut self) {
        self.remove_old_todos_wtd(Local::today());
    }

    fn remove_old_todos_wtd(&mut self, today: Date<Local>) {
        self.todos = self.todos
            .drain(..)
            .filter(|todo| { !todo.can_remove_wtd(today) })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone, Weekday};

    use crate::{Task, TdList, Todo, weekday_to_date};

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
    fn todo_displays_correctly() {
        let todo = Todo::new_undated("Todo".to_string());
        assert_eq!(todo.to_string(), "Todo (ID: 0)".to_string());
    }

    #[test]
    fn todo_for_date_tests() {
        let todo = Todo::new_specific_date("Friday".to_string(), Local.ymd(2022, 6, 10));

        let today = Local.ymd(2022, 6, 10);

        // The following 5 asserts could each be their own unit test but I'm to lazy to do it so
        // instead I just added some comments explaining the tests

        assert!(todo.for_date_wtd(today, today)); // Todo is for the given date on the same day
        assert!(todo.for_date_wtd(today, today.pred())); // Todo is for the given date before the given date
        assert!(!todo.for_date_wtd(today, today.succ())); // Todo is not for the given date after the given date
        assert!(todo.for_date_wtd(today.succ(), today.succ())); // Todo is for the following date one day after the given date
        assert!(!todo.for_date_wtd(today.succ(), today)); // Todo is not for the following date because it is already for today
    }

    #[test]
    fn todo_can_remove_returns_true_only_after_one_day_from_completion() {
        let mut todo = Todo::new_specific_date("Todo".to_string(), Local.ymd(2022, 4, 25));
        todo.set_done_wtd(true, Local.ymd(2022, 4, 26));

        assert!(!todo.can_remove_wtd(Local.ymd(2022, 4, 26)));
        assert!(todo.can_remove_wtd(Local.ymd(2022, 4, 27)));
        assert!(todo.can_remove_wtd(Local.ymd(2022, 4, 28)));
    }

    #[test]
    #[should_panic]
    fn task_new_panics_if_empty_weekday_vec() {
        Task::new("Panic!".to_string(), vec![]);
    }

    #[test]
    fn task_remove_weekday_removes_all_duplicates() {
        let mut task = Task::new("Test task".to_string(), vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Wed]);

        task.remove_weekday(Weekday::Wed);

        assert!(task.weekdays().contains(&Weekday::Mon));
        assert!(task.weekdays().contains(&Weekday::Tue));
        assert!(!task.weekdays().contains(&Weekday::Wed));
    }

    #[test]
    fn task_displays_correctly() {
        let task = Task::new("Task".to_string(), vec![Weekday::Wed]);
        assert_eq!(task.to_string(), "Task (ID: 0)".to_string());
    }

    #[test]
    fn tdlist_add_todo_updates_ids() {
        let mut list = TdList::new();

        list.add_todo(Todo::new_undated("Todo 0".to_string()));
        list.add_todo(Todo::new_undated("Todo 1".to_string()));
        list.add_todo(Todo::new_undated("Todo 2".to_string()));

        assert_eq!(list.todos()[0].id(), 0);
        assert_eq!(list.todos()[1].id(), 1);
        assert_eq!(list.todos()[2].id(), 2);
    }

    #[test]
    fn tdlist_remove_todo_updates_ids() {
        let mut list = TdList::new();

        list.add_todo(Todo::new_undated("Todo 0".to_string()));
        list.add_todo(Todo::new_undated("Todo 1".to_string()));
        list.add_todo(Todo::new_undated("Todo 2".to_string()));

        list.remove_todo(1);

        assert_eq!(list.todos()[0].id(), 0);
        assert_eq!(list.todos()[1].id(), 1);
        assert_eq!(list.todos()[0].body(), "Todo 0");
        assert_eq!(list.todos()[1].body(), "Todo 2");
        assert_eq!(list.todos().len(), 2);
    }

    #[test]
    fn tdlist_remove_todo_does_nothing_with_nonexistent_id() {
        let mut list = TdList::new();

        list.add_todo(Todo::new_undated("Todo 0".to_string()));
        list.add_todo(Todo::new_undated("Todo 1".to_string()));

        list.remove_todo(2);

        assert_eq!(list.todos()[0].id(), 0);
        assert_eq!(list.todos()[1].id(), 1);
        assert_eq!(list.todos()[0].body(), "Todo 0");
        assert_eq!(list.todos()[1].body(), "Todo 1");
    }

    #[test]
    fn tdlist_add_task_updates_ids() {
        let mut list = TdList::new();

        list.add_task(Task::new("Task 0".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 1".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 2".to_string(), vec![Weekday::Mon]));

        assert_eq!(list.tasks()[0].id(), 0);
        assert_eq!(list.tasks()[1].id(), 1);
        assert_eq!(list.tasks()[2].id(), 2);
    }

    #[test]
    fn tdlist_remove_task_updates_ids() {
        let mut list = TdList::new();

        list.add_task(Task::new("Task 0".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 1".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 2".to_string(), vec![Weekday::Mon]));

        list.remove_task(1);

        assert_eq!(list.tasks()[0].id(), 0);
        assert_eq!(list.tasks()[1].id(), 1);
        assert_eq!(list.tasks()[0].body(), "Task 0");
        assert_eq!(list.tasks()[1].body(), "Task 2");
        assert_eq!(list.tasks().len(), 2);
    }

    #[test]
    fn tdlist_remove_task_does_nothing_with_nonexistent_id() {
        let mut list = TdList::new();

        list.add_task(Task::new("Task 0".to_string(), vec![Weekday::Mon]));
        list.add_task(Task::new("Task 1".to_string(), vec![Weekday::Mon]));

        list.remove_task(2);

        assert_eq!(list.tasks()[0].id(), 0);
        assert_eq!(list.tasks()[1].id(), 1);
        assert_eq!(list.tasks()[0].body(), "Task 0");
        assert_eq!(list.tasks()[1].body(), "Task 1");
    }

    #[test]
    fn tdlist_get_todo_mut_returns_mutable() {
        let mut list = TdList::new();

        list.add_todo(Todo::new_undated("Todo".to_string()));

        assert_eq!(list.todos()[0].body(), "Todo");

        list.get_todo_mut(0).unwrap().set_body("To-Do".to_string());

        assert_eq!(list.todos()[0].body(), "To-Do");
    }

    #[test]
    fn tdlist_get_task_mut_returns_mutable() {
        let mut list = TdList::new();

        list.add_task(Task::new("Task".to_string(), vec![Weekday::Mon]));

        assert_eq!(list.tasks()[0].body(), "Task");

        list.get_task_mut(0).unwrap().set_body("Ta-Sk".to_string());

        assert_eq!(list.tasks()[0].body(), "Ta-Sk");
    }

    fn tdlist_with_done_and_undone() -> TdList {
        let mut list = TdList::new();

        list.add_todo(Todo::new_specific_date("Undone 1".to_string(), Local.ymd(2021, 4, 1)));
        list.add_todo(Todo::new_specific_date("Undone 2".to_string(), Local.ymd(2021, 3, 29)));
        list.add_todo(Todo::new_specific_date("Done 1".to_string(), Local.ymd(2021, 4, 1)));
        list.add_todo(Todo::new_specific_date("Done 2".to_string(), Local.ymd(2021, 3, 30)));

        list.get_todo_mut(2).unwrap().set_done_wtd(true, Local.ymd(2021, 4, 1));
        list.get_todo_mut(3).unwrap().set_done_wtd(true, Local.ymd(2021, 4, 1));

        list.add_task(Task::new("Undone 1".to_string(), vec![Weekday::Thu]));
        list.add_task(Task::new("Done 1".to_string(), vec![Weekday::Thu]));

        list.get_task_mut(1).unwrap().set_done(true, Local.ymd(2021, 4, 1));

        list
    }

    #[test]
    fn tdlist_undone_todos_for_date_returns_only_undone() {
        let list = tdlist_with_done_and_undone();

        let returned = list.undone_todos_for_date_wtd(Local.ymd(2021, 4, 1), Local.ymd(2021, 4, 1));

        assert!(returned.contains(&&list.todos()[0]));
        assert!(returned.contains(&&list.todos()[1]));
        assert!(!returned.contains(&&list.todos()[2]));
        assert!(!returned.contains(&&list.todos()[3]));
        assert_eq!(returned.len(), 2);
    }

    #[test]
    fn tdlist_done_todos_for_date_returns_only_done() {
        let list = tdlist_with_done_and_undone();

        let returned = list.done_todos_for_date_wtd(Local.ymd(2021, 4, 1), Local.ymd(2021, 4, 1));

        assert!(!returned.contains(&&list.todos()[0]));
        assert!(!returned.contains(&&list.todos()[1]));
        assert!(returned.contains(&&list.todos()[2]));
        assert!(returned.contains(&&list.todos()[3]));
        assert_eq!(returned.len(), 2);
    }

    #[test]
    fn tdlist_undone_tasks_for_date_returns_only_undone() {
        let list = tdlist_with_done_and_undone();

        let returned = list.undone_tasks_for_date(Local.ymd(2021, 4, 1));

        assert!(returned.contains(&&list.tasks()[0]));
        assert!(!returned.contains(&&list.tasks()[1]));
        assert_eq!(returned.len(), 1);
    }

    #[test]
    fn tdlist_done_tasks_for_date_returns_only_done() {
        let list = tdlist_with_done_and_undone();

        let returned = list.done_tasks_for_date(Local.ymd(2021, 4, 1));

        assert!(!returned.contains(&&list.tasks()[0]));
        assert!(returned.contains(&&list.tasks()[1]));
        assert_eq!(returned.len(), 1);
    }

    #[test]
    fn tdlist_remove_old_todos_removes_done_after_1_day() {
        let mut list = tdlist_with_done_and_undone();
        let list_containing_same_todos_for_eq_check = tdlist_with_done_and_undone();

        list.remove_old_todos_wtd(Local.ymd(2021, 4, 1));

        assert_eq!(list.todos(), list_containing_same_todos_for_eq_check.todos());

        list.remove_old_todos_wtd(Local.ymd(2021, 4, 2));

        assert_eq!(list.todos()[0], list_containing_same_todos_for_eq_check.todos()[0]);
        assert_eq!(list.todos()[1], list_containing_same_todos_for_eq_check.todos()[1]);
        assert_eq!(list.todos().len(), 2);
    }
}