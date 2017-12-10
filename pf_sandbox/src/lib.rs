#![feature(ord_max_min)]

extern crate bincode;
extern crate chrono;
extern crate crypto;
extern crate enum_traits;
extern crate env_logger;
extern crate getopts;
extern crate gilrs;
extern crate libusb;
extern crate lyon;
extern crate rand;
extern crate reqwest;
extern crate serde;
extern crate termcolor;
extern crate treeflection;
extern crate winit;
extern crate zip;
#[macro_use] extern crate enum_traits_macros;
#[macro_use] extern crate log;
#[macro_use] extern crate matches;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate treeflection_derive;

pub mod ai;
pub mod app;
pub mod camera;
pub mod cli;
pub mod collision;
pub mod command_line;
pub mod config;
pub mod fighter;
pub mod files;
pub mod game;
pub mod geometry;
pub mod graphics;
pub mod input;
pub mod json_upgrade;
pub mod logger;
pub mod menu;
pub mod network;
pub mod os_input;
pub mod package;
pub mod particle;
pub mod player;
pub mod replays;
pub mod results;
pub mod rules;
pub mod stage;

#[cfg(feature = "vulkan")]
#[macro_use]
extern crate vulkano;
#[cfg(feature = "vulkan")]
#[macro_use]
extern crate vulkano_shader_derive;
#[cfg(feature = "vulkan")]
extern crate vulkano_win;
#[cfg(feature = "vulkan")]
extern crate vulkano_text;
#[cfg(feature = "vulkan")]
extern crate cgmath;

#[cfg(feature = "vulkan")]
pub mod vulkan;

#[cfg(feature = "opengl")]
#[macro_use]
extern crate glium;
#[cfg(feature = "opengl")]
pub mod opengl;
#[cfg(feature = "opengl")]
pub mod opengl_buffers;
