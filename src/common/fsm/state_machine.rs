use super::state::State;
use super::state_transition::StateTransition;

pub type StateDef<Entity> = Box<dyn State<Entity = Entity> + 'static>;

pub struct StateMachine<E> {
    current_state: Option<StateDef<E>>,
    previous_state: Option<StateDef<E>>,
    global_state: Option<StateDef<E>>,
}

pub struct StateMachineBuilder<E> {
    current_state: Option<StateDef<E>>,
    global_state: Option<StateDef<E>>,
}

impl<E> StateMachineBuilder<E> {
    pub fn new() -> Self {
        StateMachineBuilder {
            current_state: None,
            global_state: None,
        }
    }

    pub fn set_initial_state(mut self, state: StateDef<E>) -> Self {
        self.current_state = Some(state);
        self
    }

    pub fn set_global_state(mut self, state: StateDef<E>) -> Self {
        self.global_state = Some(state);
        self
    }

    pub fn build(self) -> StateMachine<E> {
        StateMachine {
            global_state: self.global_state,
            current_state: self.current_state,
            previous_state: None,
        }
    }
}

impl<E> StateMachine<E> {
    pub fn update(&mut self, entity: &mut E) {
        let global_transition = match &mut self.global_state {
            Some(state) => state.execute(entity),
            None => StateTransition::None,
        };
        self.handle_transition(global_transition, entity);

        let current_transition = match &mut self.current_state {
            Some(state) => state.execute(entity),
            None => StateTransition::None,
        };
        self.handle_transition(current_transition, entity);
    }

    fn handle_transition(&mut self, transition: StateTransition<E>, entity: &mut E) {
        match transition {
            StateTransition::None => {}
            StateTransition::Push(state) => {
                self.exit_current_state(entity);
                self.previous_state = self.current_state.take();
                self.current_state = Some(state);
                self.enter_current_state(entity);
            }
            StateTransition::Pop() => {
                self.exit_current_state(entity);
                self.current_state = self.previous_state.take();
                self.enter_current_state(entity);
            }
            StateTransition::Switch(state) => {
                self.exit_current_state(entity);
                self.current_state = Some(state);
                self.enter_current_state(entity);
            }
            StateTransition::Exit() => {}
        }
    }

    fn exit_current_state(&mut self, entity: &mut E) {
        match self.current_state {
            Some(ref mut state) => state.exit(entity),
            _ => (),
        }
    }

    fn enter_current_state(&mut self, entity: &mut E) {
        match self.current_state {
            Some(ref mut state) => state.enter(entity),
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::fsm::state::State;
    use crate::common::fsm::state_machine::StateMachineBuilder;
    use crate::common::fsm::state_transition::StateTransition;

    struct TestEntity {
        pub state1: i32,
        pub state2: i32,
    }

    struct TestState1;

    impl State for TestState1 {
        type Entity = TestEntity;

        fn new() -> Box<Self> {
            Box::new(TestState1 {})
        }

        fn enter(&mut self, entity: &mut Self::Entity) {
            println!("Entering state 1");
            entity.state1 += 1;
        }

        fn execute(&mut self, entity: &mut Self::Entity) -> StateTransition<TestEntity> {
            println!("Executing state 1");
            entity.state1 += 20;
            StateTransition::Switch(TestState2::new())
        }

        fn exit(&mut self, entity: &mut Self::Entity) {
            println!("Exiting state 1");
            entity.state1 *= 10;
        }
    }

    struct TestState2;

    impl State for TestState2 {
        type Entity = TestEntity;

        fn new() -> Box<Self>
        where
            Self: Sized,
        {
            Box::new(TestState2 {})
        }

        fn enter(&mut self, entity: &mut Self::Entity) {
            println!("Entering state 2");
            entity.state2 += 2;
        }

        fn execute(&mut self, entity: &mut Self::Entity) -> StateTransition<TestEntity> {
            println!("Executing state 2");
            entity.state2 += 30;
            StateTransition::None
        }

        fn exit(&mut self, entity: &mut Self::Entity) {
            println!("Exiting state 2");
            entity.state2 *= 10;
        }
    }

    #[test]
    fn test_run() {
        let mut machine = StateMachineBuilder::<TestEntity>::new()
            .set_initial_state(Box::new(TestState1 {}))
            .build();
        let mut entity = TestEntity {
            state1: 0,
            state2: 0,
        };
        machine.update(&mut entity);
        machine.update(&mut entity);
        assert_eq!(entity.state1, 200);
        assert_eq!(entity.state2, 32);
    }
}
