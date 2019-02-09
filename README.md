# guitar-exercises
A guitar exercise generator, altho probably can be used for other
stringed instruments.

## Progression
The exercises are permutations over a scale.
A table shows the notes over a fretboard, hopefully the width is based
on the frequency. Later on SVG can be used, maybe some midi audio.
By default, the C Natural Scale will be used, it should show a progression
to understand how to start using other scales.

## Why
This is in the interest of music theory (the typing system)
And the lack of an online tool that just gives me some
guidelines on what to work on, altho I have played guitar for a number of years
I have close to zero knowledge on the physics of the guitar and music theory.
This is an exploration on those unknown areas. I will make many assumptions
that will probably turn out to be wrong, and that's the whole idea.

It is difficult for me to keep a methodic progression on scales,
picking techniques, etc.

## Cookies
If ever, cookies will be used only to keep track of local progress,
customization and provide next exercises.
One of the main reasons I built this is that I'm tired of click-bait,
ads, tracking, etc.
You can run this in your machine with docker, or maybe use
digitalocean droplets. Maybe an HTML only version can be generated.

## Running locally
TBD: DockerHub
`cargo run` should be enough, then on a browser localhost:8080

## Tooling
Using JenkinsX, Rust and Rocket.
Fork it, cd into clone and `jx import` if you want to change things, PRs are welcome
