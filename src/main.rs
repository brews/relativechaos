use bevy::app::{AppExit, ScheduleRunnerPlugin};
use bevy::prelude::*;
use bevy_ratatui::event::KeyMessage;
use bevy_ratatui::{RatatuiContext, RatatuiPlugins};
use crossterm::event::KeyCode;
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

/// System detecting user input, such as key presses.
///
/// Run on Update schedule.
fn input_system(mut messages: MessageReader<KeyMessage>, mut exit: MessageWriter<AppExit>) {
    for message in messages.read() {
        if let KeyCode::Char('q') = message.code {
            exit.write_default();
        }
    }
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
