#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::io::BufReader;
use id3::{Tag, TagLike};

use clap::Parser;
use id3::frame::Picture;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, SharedString};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File being loaded in
    #[arg(short, long)]
    file: String,
}


slint::include_modules!();

#[derive(Debug, PartialOrd, PartialEq)]
enum PlayerState {
    Playing,
    Paused,
    Stopped,
}

impl PlayerState {
    fn as_str(&self) -> &str {
        match self {
            PlayerState::Playing => "playing",
            PlayerState::Paused => "paused",
            PlayerState::Stopped => "stopped",
        }
    }
}

fn apply_action(action: &str, sink: &rodio::Sink) {
    match action {
        "play" => sink.play(),
        "pause" => sink.pause(),
        "stop" => sink.stop(),
        _ => {}
    }
}

fn load_sink(handle: &rodio::OutputStreamHandle, file: &str) -> rodio::Sink {
    let file = std::fs::File::open(file).unwrap();
    let sink = rodio::Sink::try_new(handle).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
    sink.pause();
    sink
}

fn render_song_tags(ui: &AppWindow, tag: &Tag) {
    ui.set_song_title(tag.title().unwrap_or("Unknown Title").into());
    ui.set_song_artist(tag.artist().unwrap_or("Unknown Artist").into());
    ui.set_song_album(tag.album().unwrap_or("Unknown Album").into());

    let cover_image_tag = tag.pictures().next();

    if let Some(cover_image_tag) = cover_image_tag {
        render_cover_image(ui, cover_image_tag);
    }
}

fn render_cover_image(ui: &AppWindow, cover_image: &Picture) {
    let image = image::load_from_memory(&cover_image.data).unwrap();
    let rgba = image.to_rgba8();
    let raw = rgba.as_raw();
    ui.set_cover({
        Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            raw,
            image.width(),
            image.height(),
        ))
    });
}

fn play_button(ui: &AppWindow, sink: &rodio::Sink) {
    if ui.get_player_status().as_str() == PlayerState::Playing.as_str() {
        ui.set_player_status(PlayerState::Paused.as_str().into());
        sink.pause();
    } else {
    ui.set_player_status(PlayerState::Playing.as_str().into());
    sink.play();
        }
}

fn pause_button(ui: &AppWindow, sink: &rodio::Sink) {
    ui.set_player_status(PlayerState::Paused.as_str().into());
    sink.pause();
}

fn stop_button(ui: &AppWindow, sink: &rodio::Sink) {
    ui.set_player_status(PlayerState::Stopped.as_str().into());
    sink.stop();
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let ui = AppWindow::new()?;

    ui.set_player_status(PlayerState::Stopped.as_str().into());

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = load_sink(&handle, &args.file);

    let tag = Tag::read_from_path(&args.file)?;
    render_song_tags(&ui, &tag);

    ui.on_emit({
        let ui = ui.as_weak();
        move |action: SharedString| {
            let ui = ui.upgrade().unwrap();

            match action.as_str() {
                "play" | "pause" => { play_button(&ui, &sink); }
                "stop" => { stop_button(&ui, &sink); }
                _ => {}
            }
        }
    });

    ui.run()?;

    Ok(())
}
