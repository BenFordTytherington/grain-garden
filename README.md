# Grain Garden
Grain Garden is a granular synthesizer controlled by a procedurally generated plant.
Built with Rust, Egui and Rodio.

## Sequencer
Currently, the sequencer is in its most basic form, where a leaf from the tree is randomly selected, and this corresponds with a grain.
The height, relative to the max height of the tree is the grain start position inside the spawning window.
The x position corresponds with panning the grain. The granular control module has a density parameter, specifying grain spawn rate in Hz.

## Screenshots
The Ui for Grain Garden currently looks like this.
![Grain Garden UI](assets/Ui_2.png)

### Some examples of procedural plants

<details>
    <summary>Ferns</summary>

![Tree 1](assets/Tree1.png)
![Tree 2](assets/Tree2.png)
![Tree 3](assets/Tree3.png)
![Tree 4](assets/Tree4.png)

</details>

<details>
    <summary>Trees</summary>

![Tree 1](assets/Tree5.png)
![Tree 2](assets/Tree6.png)
![Tree 3](assets/Tree7.png)
![Tree 4](assets/Tree8.png)
![Tree 5](assets/Tree9.png)
![Tree 6](assets/Tree10.png)
![Tree 7](assets/Tree11.png)
![Tree 8](assets/Tree12.png)
![Tree 9](assets/Tree13.png)
![Tree 10](assets/Tree14.png)

</details>