use deckgym::{
    actions::{Action, SimpleAction},
    card_ids::CardId,
    database::get_card_by_enum,
    models::{EnergyType, PlayedCard},
    test_support::get_initialized_game,
};

fn attack(actor: usize, attack_index: usize) -> Action {
    Action {
        actor,
        action: SimpleAction::Attack(attack_index),
        is_stack: false,
    }
}

fn played_card_with_base_hp(card_id: CardId, base_hp: u32) -> PlayedCard {
    let card = get_card_by_enum(card_id);
    PlayedCard::new(card, 0, base_hp, vec![], false, vec![])
}

#[test]
fn test_quick_straight_ignores_fighting_weakness() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::B1124HitmonchanEx)
            .with_energy(vec![EnergyType::Fighting])],
        vec![PlayedCard::from_id(CardId::A1094Pikachu)],
    );
    state.current_player = 0;
    game.set_state(state);

    game.apply_action(&attack(0, 0));

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        10,
        "Quick Straight should deal 50 to Fighting-weak Pikachu, not 70"
    );
}

#[test]
fn test_quick_straight_keeps_lucario_boost_without_weakness() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![
            PlayedCard::from_id(CardId::B1124HitmonchanEx).with_energy(vec![EnergyType::Fighting]),
            PlayedCard::from_id(CardId::A2092Lucario),
        ],
        vec![played_card_with_base_hp(CardId::A1094Pikachu, 100)],
    );
    state.current_player = 0;
    game.set_state(state);

    game.apply_action(&attack(0, 0));

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        30,
        "Quick Straight with Fighting Coach should deal 70, not 90"
    );
}

#[test]
fn test_quick_straight_keeps_defender_reduction() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::B1124HitmonchanEx)
            .with_energy(vec![EnergyType::Fighting])],
        vec![PlayedCard::from_id(CardId::A1211Snorlax)
            .with_tool(get_card_by_enum(CardId::B1219HeavyHelmet))],
    );
    state.current_player = 0;
    game.set_state(state);

    game.apply_action(&attack(0, 0));

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        120,
        "Quick Straight should deal 30 through Heavy Helmet: 50 - 20"
    );
}

#[test]
fn test_quick_straight_ignores_bounded_field_weakness_multiplier() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::B1124HitmonchanEx)
            .with_energy(vec![EnergyType::Fighting])],
        vec![PlayedCard::from_id(CardId::A1094Pikachu)],
    );
    state.current_player = 0;
    state.active_stadium = Some(get_card_by_enum(CardId::B3155BoundedField));
    game.set_state(state);

    game.apply_action(&attack(0, 0));

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        10,
        "Bounded Field should not add or multiply Weakness for Quick Straight"
    );
}

#[test]
fn test_copied_quick_straight_ignores_weakness() {
    let mut game = get_initialized_game(0);
    let mut state = game.get_state_clone();

    state.set_board(
        vec![PlayedCard::from_id(CardId::A1a032MewEx).with_energy(vec![
            EnergyType::Psychic,
            EnergyType::Psychic,
            EnergyType::Psychic,
        ])],
        vec![PlayedCard::from_id(CardId::B1124HitmonchanEx)
            .with_energy(vec![EnergyType::Fighting])],
    );
    state.current_player = 0;
    state.turn_count = 3;
    game.set_state(state);

    game.apply_action(&attack(0, 1));

    let (_actor, choices) = game.get_state_clone().generate_possible_actions();
    let copied_quick_straight = choices
        .iter()
        .find(|action| {
            matches!(
                action.action,
                SimpleAction::UseCopiedAttack {
                    source_player: 1,
                    source_in_play_idx: 0,
                    attack_index: 0,
                    require_attacker_energy_match: false,
                }
            )
        })
        .expect("Genome Hacking should offer copied Quick Straight")
        .clone();

    game.apply_action(&copied_quick_straight);

    let state = game.get_state_clone();
    assert_eq!(
        state.get_active(1).get_remaining_hp(),
        80,
        "Copied Quick Straight should deal 50 to Psychic-weak Hitmonchan ex, not 70"
    );
}
