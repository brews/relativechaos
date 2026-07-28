use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use bevy_ratatui::{RatatuiContext, RatatuiPlugins};
use crossterm::event::KeyCode;
use rand::RngExt;
use ratatui::layout::{Constraint, Layout};
use ratatui::prelude::Color;
use ratatui::style::Stylize;
use ratatui::symbols::Marker;
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::Block;
use ratatui::widgets::canvas::{Canvas, Points};

/// World length along the x axis.
const WORLD_X: i32 = 80;
/// World length along the y axis.
const WORLD_Y: i32 = 50;

fn main() {
    let frame_time = std::time::Duration::from_secs_f32(1. / 60.);
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(frame_time)),
            RatatuiPlugins::default(),
            // HelloPlugin,
        ))
        .add_systems(Update, render_system)
        .add_systems(Update, input_system)
        .add_systems(Update, leftwalker)
        .add_systems(Startup, add_characters)
        .add_observer(player_mover)
        .init_resource::<Map>()
        .run();
}

/// System to render the UI.
///
/// Run on Update schedule.
fn render_system(
    mut context: ResMut<RatatuiContext>,
    query: Query<(&Renderable, &Position)>,
) -> Result {
    context.draw(|frame| {
        // Screen layouts for UI.
        let horizontal = Layout::horizontal([Constraint::Percentage(100)]).spacing(1);
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
        let [top, main] = frame.area().layout(&vertical);
        let [area] = main.layout(&horizontal);

        // To see layout areas.
        frame.render_widget(Block::bordered(), top);
        frame.render_widget(Block::bordered(), main);
        frame.render_widget(Block::bordered(), area);

        // UI title.
        let title = TextLine::from_iter([
            Span::from("relativechaos").bold(),
            Span::from(" (press 'q' to quit)"),
        ]);
        frame.render_widget(title.centered(), top);

        // Draw renderable entitiles on map.
        // Note: the origin is the lower left corner of the canvas.
        let canvas = Canvas::default()
            .x_bounds([0.0, WORLD_X as f64])
            .y_bounds([0.0, WORLD_Y as f64])
            .paint(|ctx| {
                query.iter().for_each(|(renderable, position)| {
                    ctx.marker(Marker::Custom(renderable.glyph));
                    ctx.draw(&Points::new(
                        &[(position.x as f64, position.y as f64)],
                        renderable.color,
                    ))
                })
            });
        frame.render_widget(canvas, area);
    })?;

    Ok(())
}

/// System detecting user input such as key presses.
///
/// Run on Update schedule.
fn input_system(
    mut commands: Commands,
    mut keyboard_message: MessageReader<KeyMessage>,
    mut exit: MessageWriter<AppExit>,
) {
    keyboard_message
        .read()
        .for_each(|message| match message.code {
            KeyCode::Char('q') => _ = exit.write_default(),
            KeyCode::Up => commands.trigger(TryMovePlayer(Direction::Up)),
            KeyCode::Down => commands.trigger(TryMovePlayer(Direction::Down)),
            KeyCode::Left => commands.trigger(TryMovePlayer(Direction::Left)),
            KeyCode::Right => commands.trigger(TryMovePlayer(Direction::Right)),
            _ => {}
        });
}

/// Event indicating the player entity is attempting to move in a direction.
#[derive(Event)]
struct TryMovePlayer(Direction);

/// Movement of an entity in one of the four cardinal directions.
#[derive(PartialEq, Debug)]
enum Direction {
    /// Movement "north", towards the top of the map.
    Up,
    /// Movement "south", towards the bottom of the map.
    Down,
    /// Movement "west", to port.
    Left,
    /// Movement "east", to starboard.
    Right,
}

/// Observer to handle player movement events, changing the player's position, if possible.
///
/// This changes map position based on the direction the player is attempting to move, but
/// prevents movement if there is an obstruction or map edge.
fn player_mover(attempted_move: On<TryMovePlayer>, mut player: Query<&mut Position, With<Player>>) {
    let mut player_position = player
        .single_mut()
        .expect("Either none or multiple players exist in the world and this should never happen");

    // Translate direction into velocity.
    let mut delta_x: i32 = 0;
    let mut delta_y: i32 = 0;
    match attempted_move.0 {
        Direction::Up => delta_y = 1,
        Direction::Down => delta_y = -1,
        Direction::Left => delta_x = -1,
        Direction::Right => delta_x = 1,
    }

    // Apply velocity to change the player's position, but only within map's bounds.
    player_position.y = (player_position.y + delta_y).clamp(0, WORLD_Y - 1);
    player_position.x = (player_position.x + delta_x).clamp(0, WORLD_X - 1);
}

/// Component for entities that have a position on the map.
#[derive(Component)]
struct Position {
    /// Position along the horizontal axis.
    x: i32,
    /// Position along the vertical axis.
    y: i32,
}

/// Component for entities that can be rendered on the display.
#[derive(Component)]
struct Renderable {
    /// Character or glyph to be rendered.
    glyph: char,
    /// Color of the rendered glyph.
    color: Color,
}

/// Component flagging entities that move leftwise.
#[derive(Component)]
struct LeftMover;

/// Component indicating the entity is the player character.
#[derive(Component, Debug)]
struct Player;

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn hello_world() {
    println!("hello world!");
}

/// System to add characters to world.
///
/// Run on game startup.
fn add_characters(mut commands: Commands) {
    // Spawn the player character.
    commands.spawn((
        Renderable {
            glyph: '@',
            color: Color::Yellow,
        },
        Position { x: 40, y: 25 },
        Player,
    ));

    // Spawn some intimidating looking characters.
    for i in 0..10 {
        commands.spawn((
            Renderable {
                glyph: '☺',
                color: Color::Red,
            },
            Position { x: i * 7, y: 20 },
            LeftMover {},
        ));
    }
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn greet_people(
    mut context: ResMut<RatatuiContext>,
    time: Res<Time>,
    mut timer: ResMut<GreetTimer>,
    query: Query<&Name, With<Person>>,
) -> Result {
    // Update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone.
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            context.draw(|frame| {
                let msg_str = format!("hello {}!", name.0);
                let text = ratatui::text::Text::raw(msg_str);
                frame.render_widget(text, frame.area());
            })?;
        }
    }

    Ok(())
}

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // no need to change other names.
        }
    }
}

/// System moving [LeftMover] entities to the left.
fn leftwalker(mut query: Query<&mut Position, With<LeftMover>>) {
    query.iter_mut().for_each(|mut component| {
        component.x -= 1;
        if component.x < 0 {
            component.x = WORLD_X - 1;
        }
    })
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_characters);
        app.add_systems(Update, (hello_world, (update_people, greet_people).chain()));
    }
}

#[derive(Resource)]
struct GreetTimer(Timer);

/// Individual map tile types.
#[derive(PartialEq, Clone)]
enum MapTile {
    Wall,
    Floor,
}

#[derive(Resource)]
struct Map(Vec<MapTile>);

impl FromWorld for Map {
    fn from_world(_world: &mut World) -> Self {
        generate_map()
    }
}

/// Translate position x, y grid coordinates to index to the corresponding map tile.
fn xy2idx(x: i32, y: i32) -> usize {
    ((y * WORLD_X) + x) as usize
}

/// Use thread-local RNG to generate a map.
fn generate_map() -> Map {
    // This is straight from Wolverson 2019, section 3.2.
    let mut map = vec![MapTile::Floor; (WORLD_X * WORLD_Y) as usize];

    // Boundary walls
    for x in 0..WORLD_X {
        map[xy2idx(x, 0)] = MapTile::Wall;
        map[xy2idx(x, WORLD_Y - 1)] = MapTile::Wall;
    }
    for y in 0..WORLD_Y {
        map[xy2idx(0, y)] = MapTile::Wall;
        map[xy2idx(WORLD_X - 1, y)] = MapTile::Wall;
    }

    // Randomly add some walls.
    let mut rng = rand::rng();
    for _ in 0..400 {
        let x = rng.random_range(0..(WORLD_X - 1));
        let y = rng.random_range(0..(WORLD_Y - 1));
        let idx = xy2idx(x, y);
        map[idx] = MapTile::Wall;
    }

    // Always make player starting position a floor, so player doesn't spawn in a wall.
    // Player character starting map index in middle of map.
    let starting_idx = xy2idx(WORLD_X / 2, WORLD_Y / 2);
    map[starting_idx] = MapTile::Floor;

    Map(map)
}
