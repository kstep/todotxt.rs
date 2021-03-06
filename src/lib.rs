extern crate chrono;

use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::{self, Write};
pub use chrono::NaiveDate as Date;

#[derive(Debug, Eq, PartialEq)]
pub enum Recurrence {
    Daily(bool, u16),
    BDaily(bool, u16),
    Monthly(bool, u16),
    Weekly(bool, u16),
    Yearly(bool, u16),
}

impl FromStr for Recurrence {
    type Err = ();
    fn from_str(s: &str) -> Result<Recurrence, ()> {
        use self::Recurrence::*;

        let hard = s.starts_with("+");

        s[hard as usize..s.len() - 1].parse::<u16>().map_err(|_| ()).and_then(|num| {
            Ok(match &s[s.len() - 1..] {
                "d" => Daily(hard, num),
                "b" => BDaily(hard, num),
                "m" => Monthly(hard, num),
                "w" => Weekly(hard, num),
                "y" => Yearly(hard, num),
                _ => return Err(()),
            })
        })
    }
}

impl fmt::Display for Recurrence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Recurrence::*;
        try!(f.write_str("rec:"));
        match *self {
            Daily(hard, num) => {
                if hard {
                    try!(f.write_char('+'));
                }
                try!(write!(f, "{}d", num));
            }
            BDaily(hard, num) => {
                if hard {
                    try!(f.write_char('+'));
                }
                try!(write!(f, "{}b", num));
            }
            Monthly(hard, num) => {
                if hard {
                    try!(f.write_char('+'));
                }
                try!(write!(f, "{}m", num));
            }
            Weekly(hard, num) => {
                if hard {
                    try!(f.write_char('+'));
                }
                try!(write!(f, "{}w", num));
            }
            Yearly(hard, num) => {
                if hard {
                    try!(f.write_char('+'));
                }
                try!(write!(f, "{}y", num));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Task {
    pub subject: String,
    pub priority: u8,
    pub create_date: Option<Date>,
    pub finish_date: Option<Date>,
    pub finished: bool,
    pub threshold_date: Option<Date>,
    pub due_date: Option<Date>,
    pub recurrence: Option<Recurrence>,
    pub contexts: Vec<String>,
    pub projects: Vec<String>,
    pub hashtags: Vec<String>,
    pub tags: HashMap<String, String>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.finished {
            try!(f.write_str("x "));
            if let Some(ref finish_date) = self.finish_date {
                try!(write!(f, "{} ", finish_date));
            }
        }

        if self.priority < 26 {
            try!(write!(f, "({}) ", (self.priority + b'A') as char));
        }

        if let Some(ref create_date) = self.create_date {
            try!(write!(f, "{} ", create_date));
        }

        try!(f.write_str(&self.subject));

        if let Some(ref date) = self.due_date {
            try!(write!(f, " due:{}", date));
        }
        if let Some(ref date) = self.threshold_date {
            try!(write!(f, " t:{}", date));
        }
        if let Some(ref rec) = self.recurrence {
            try!(write!(f, " rec:{}", rec));
        }
        for (tag, value) in &self.tags {
            try!(write!(f, " {}:{}", tag, value));
        }

        Ok(())
    }
}

impl FromStr for Task {
    type Err = ();
    fn from_str(mut s: &str) -> Result<Task, ()> {
        // parse finish state
        let (finished, mut finish_date) = if s.starts_with("x ") {
            s = &s[2..];
            (true, s[..10].parse::<Date>().ok())
        } else {
            (false, None)
        };

        if finish_date.is_some() {
            s = &s[11..];
        }

        // parse priority
        let priority = if s.starts_with("(") && &s[2..4] == ") " {
            match s.as_bytes()[1] {
                p @ b'A'...b'Z' => {
                    s = &s[4..];
                    p - b'A'
                }
                _ => 26,
            }
        } else {
            26
        };

        // parse creation date
        let mut create_date = match s[..10].parse::<Date>() {
            Ok(date) => {
                s = &s[11..];
                Some(date)
            }
            Err(_) => None,
        };

        // If creation date follows finished mark, it can be parsed as a finish date,
        // which is wrong. Note finish state part and creation date can be separated
        // by priority, so this confusion is possible only if no priority given.
        if priority == 26 && finish_date.is_some() && create_date.is_none() {
            create_date = finish_date;
            finish_date = None;
        }

        let buf = s;

        // Subject is the line with technical info removed (priority, creation date and finish state, tags).
        let mut subject = Vec::new();

        // FSM to parse line for tags, contexts and projects.

        #[derive(Copy, Clone, Eq, PartialEq)]
        enum St {
            Init,
            Ctx(usize),
            Prj(usize),
            Hash(usize),
            Tag0(usize),
            Tag1(usize, usize),
        }
        let mut state = St::Init;
        let mut contexts = Vec::new();
        let mut projects = Vec::new();
        let mut hashtags = Vec::new();
        let mut tags = HashMap::new();

        // Some known tags: threshold date (`t:`), due date (`due:`) and recurrence (`rec:`).
        let mut threshold_date = None;
        let mut due_date = None;
        let mut recurrence = None;

        for (i, c) in buf.bytes().enumerate() {
            let new_state = match (c, state) {
                (b'@', St::Init) => St::Ctx(i),
                (b'+', St::Init) => St::Prj(i),
                (b'#', St::Init) => St::Hash(i),
                (b'a'...b'z', St::Init) => St::Tag0(i),
                (b':', St::Tag0(j)) => St::Tag1(j, i),
                (b' ', St::Tag0(_)) => St::Init,
                (b' ', St::Ctx(j)) => {
                    if i - j > 1 {
                        contexts.push(buf[j + 1..i].to_owned());
                    }
                    St::Init
                }
                (b' ', St::Prj(j)) => {
                    if i - j > 1 {
                        projects.push(buf[j + 1..i].to_owned());
                    }
                    St::Init
                }
                (b' ', St::Hash(j)) => {
                    if i - j > 1 {
                        hashtags.push(buf[j + 1..i].to_owned());
                    }
                    St::Init
                }
                (b' ', St::Tag1(j, k)) => {
                    if i - k > 1 {
                        match &buf[j..k] {
                            "rec" => {
                                recurrence = buf[k + 1..i].parse::<Recurrence>().ok();
                            }
                            "due" => {
                                due_date = buf[k + 1..i].parse::<Date>().ok();
                            }
                            "t" => {
                                threshold_date = buf[k + 1..i].parse::<Date>().ok();
                            }
                            tag => {
                                tags.insert(tag.to_owned(), buf[k + 1..i].to_owned());
                            }
                        }
                    }
                    St::Init
                }
                _ => state,
            };

            if new_state == St::Init {
                match state {
                    St::Tag0(j) | St::Hash(j) | St::Prj(j) | St::Ctx(j) => {
                        subject.extend(&buf.as_bytes()[j..i]);
                    }
                    St::Init => subject.push(buf.as_bytes()[i]),
                    _ => {}
                }
            }

            state = new_state;
        }

        // Check final state, so tags at the end of line are also parsed.
        match state {
            St::Tag1(j, k) => {
                tags.insert(buf[j..k].to_owned(), buf[k + 1..].to_owned());
            }
            St::Prj(j) => {
                projects.push(buf[j + 1..].to_owned());
                subject.extend(&buf.as_bytes()[j..]);
            }
            St::Ctx(j) => {
                contexts.push(buf[j + 1..].to_owned());
                subject.extend(&buf.as_bytes()[j..]);
            }
            St::Hash(j) => {
                hashtags.push(buf[j + 1..].to_owned());
                subject.extend(&buf.as_bytes()[j..]);
            }
            St::Tag0(j) => {
                subject.extend(&buf.as_bytes()[j..]);
            }
            _ => {}
        }

        let subject = String::from_utf8(subject).unwrap_or_else(|_| s.to_owned());

        Ok(Task {
            subject: subject,
            priority: priority,
            create_date: create_date,
            finish_date: finish_date,
            finished: finished,
            threshold_date: threshold_date,
            due_date: due_date,
            recurrence: recurrence,
            contexts: contexts,
            projects: projects,
            hashtags: hashtags,
            tags: tags,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Date, Recurrence, Task};

    #[test]
    fn it_works() {
        let todo_item = "(A) 2016-03-24 22:00 сходить на занятие в @microfon rec:+1w \
                         due:2016-04-05 t:2016-04-05 at:20:00";
        let task = todo_item.parse::<Task>().unwrap();
        assert_eq!(task.subject, "22:00 сходить на занятие в @microfon");
        assert_eq!(task,
                   Task {
                       subject: "22:00 сходить на занятие в @microfon".to_owned(),
                       create_date: Some(Date::from_ymd(2016, 3, 24)),
                       priority: 0,
                       recurrence: Some(Recurrence::Weekly(true, 1)),
                       due_date: Some(Date::from_ymd(2016, 4, 5)),
                       threshold_date: Some(Date::from_ymd(2016, 4, 5)),
                       contexts: vec!["microfon".to_owned()],
                       tags: vec![("at".to_owned(), "20:00".to_owned())].into_iter().collect(),
                       ..Task::default()
                   });

        let todo_item = "2016-03-27 сменить загранпаспорт due:2020-08-14 t:2020-04-14 +документы";
        let task = todo_item.parse::<Task>().unwrap();
        assert_eq!(task.subject, "сменить загранпаспорт +документы");
        assert_eq!(task,
                   Task {
                       subject: "сменить загранпаспорт +документы".to_owned(),
                       create_date: Some(Date::from_ymd(2016, 3, 27)),
                       priority: 26,
                       due_date: Some(Date::from_ymd(2020, 8, 14)),
                       threshold_date: Some(Date::from_ymd(2020, 4, 14)),
                       projects: vec!["документы".to_owned()],
                       ..Task::default()
                   });

        let todo_item = "x 2016-03-27 сменить загранпаспорт due:2020-08-14 t:2020-04-14 +документы";
        let task = todo_item.parse::<Task>().unwrap();
        assert_eq!(task.subject, "сменить загранпаспорт +документы");
        assert_eq!(task,
                   Task {
                       subject: "сменить загранпаспорт +документы".to_owned(),
                       create_date: Some(Date::from_ymd(2016, 3, 27)),
                       priority: 26,
                       due_date: Some(Date::from_ymd(2020, 8, 14)),
                       threshold_date: Some(Date::from_ymd(2020, 4, 14)),
                       projects: vec!["документы".to_owned()],
                       finished: true,
                       ..Task::default()
                   });
    }
}
