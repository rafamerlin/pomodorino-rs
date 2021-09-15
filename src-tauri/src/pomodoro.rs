use crossbeam::channel::Sender;

#[derive(Clone, Debug)]
pub enum PomodoroState {
  Clear,
  Running(f32, usize),
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
    self.state = PomodoroState::Running(length as f32 * TIME_MULTIPLIER + 1.0, FORCE_MINUTE_UPDATE);
    self
  }

  pub fn clear(&mut self) -> () {
    if let PomodoroState::Completed = self.state {
      self.state = PomodoroState::Clear;
      self.tx.send(self.state.clone()).unwrap();
    }
  }

  pub fn tick(&mut self) -> &mut Pomodoro {
    println!("Tick");
    if let PomodoroState::Running(t, m) = self.state {
      let new_t = t - 1.00;
      if new_t == 0.00 {
        self.state = PomodoroState::Completed;
        self.tx.send(self.state.clone()).unwrap();
      } else {
        let minutes = if new_t % TIME_MULTIPLIER <= 0.0 {
          (new_t / TIME_MULTIPLIER).ceil() as usize
        } else {
          m
        };

        self.state = PomodoroState::Running(new_t, minutes);
        if minutes != m || TIME_MULTIPLIER == 1.0 {
          self.tx.send(self.state.clone()).unwrap();
        }
      }
    }
    self
  }
}
