use crossbeam::channel::Sender;

#[derive(Clone, Debug)]
pub enum PomodoroState {
  Clear,
  Running(f32, usize, Option<String>),
  Completed(Option<String>),
}

enum InfoState {
  Running(usize),
  Completed,
}

pub struct Pomodoro {
  state: PomodoroState,
  tx: Sender<PomodoroState>,
}

const TIME_MULTIPLIER: f32 = 60.0;

impl Pomodoro {
  pub fn new(tx: Sender<PomodoroState>) -> Self {
    Self {
      state: PomodoroState::Clear,
      tx,
    }
  }

  pub fn start(&mut self, length: usize) -> &mut Pomodoro {
    const FORCE_MINUTE_UPDATE: usize = 9999;
    self.state = PomodoroState::Running(
      length as f32 * TIME_MULTIPLIER + 1.0,
      FORCE_MINUTE_UPDATE,
      Some(Pomodoro::generate_info(InfoState::Running(length))),
    );
    self
  }

  pub fn cancel(&mut self) -> () {
    self.state = PomodoroState::Clear;
    self.tx.send(self.state.clone()).unwrap();
  }

  pub fn clear(&mut self) -> () {
    if let PomodoroState::Completed(_) = self.state {
      self.state = PomodoroState::Clear;
      self.tx.send(self.state.clone()).unwrap();
    }
  }

  pub fn tick(&mut self) -> &mut Pomodoro {
    if let PomodoroState::Running(t, m, info) = &self.state {
      let new_t = t - 1.00;
      let m = *m;
      if new_t == 0.00 {
        self.state = PomodoroState::Completed(Some(Pomodoro::generate_info(InfoState::Completed)));
        self.tx.send(self.state.clone()).unwrap();
      } else {
        let minutes = if new_t % TIME_MULTIPLIER <= 0.0 {
          (new_t / TIME_MULTIPLIER).ceil() as usize
        } else {
          m
        };

        let clean_info_after_first_use = info.is_some();
        self.state = PomodoroState::Running(new_t, minutes, info.to_owned());
        if minutes != m || TIME_MULTIPLIER == 1.0 {
          self.tx.send(self.state.clone()).unwrap();
          if clean_info_after_first_use {
            //Clean the info after the first use so we don't update the menu every minute.
            self.state = PomodoroState::Running(new_t, minutes, None);
          }
        }
      }
    }
    self
  }

  fn generate_info(state: InfoState) -> String {
    let current_time = chrono::offset::Local::now().format("%H:%M:%S");
    let response = match state {
      InfoState::Running(len) => format!("{} Pomo Started at {}", len, current_time),
      InfoState::Completed => format!("Pomo Finished at {}", current_time),
    };

    response
  }
}
