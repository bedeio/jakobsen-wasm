const { Vec2D, Scene, Cmp } = require('../pkg/wasm_verlet');
const { memory } = require('../pkg/wasm_verlet_bg');

// config
let animationId = null;

document.addEventListener('onload', init());

function initRope(scene) {
    let gravity = new Vec2D(0, 100);
    let anchor = new Vec2D(100, 100);
    console.log('Scene:', scene);

    let SEG_LEN = 16;
    let SEG_NUM = 8;
    for (let i = 0; i < SEG_NUM; i++) {
        console.log('i:', i);
        let x = i * SEG_LEN + anchor.x;
        let y = anchor.y;

        let plos = new Vec2D(x, y);
        console.log('Pos:', plos);
        scene.add_particle(plos, plos, 0.1);
        scene.set_force(i, gravity);

        if (i == 0) {
            scene.add_fixed_constraint(i, anchor, SEG_LEN, Cmp.Less);
        } else {
            scene.add_point_constraint(i, i - 1, SEG_LEN, Cmp.Less);
        }
    }
}

function initCloth(scene) {
    let gravity = new Vec2D(0, 200);
    let anchor = new Vec2D(100, 100);
    console.log('Scene:', scene);

    let SEG_LEN = 12;
    let COLS = 15;
    let ROWS = 10;
    for (let i = 0; i < ROWS; i++) {
        for (let j = 0; j < COLS; j++) {
            let x = j * SEG_LEN + anchor.x;
            let y = anchor.y;
            let pos = new Vec2D(x, y);
            scene.add_particle(pos, pos, 1);
            scene.set_force(i * COLS + j, gravity);
        }
    }

    for (let i = 0; i < ROWS; i++) {
        for (let j = 0; j < COLS; j++) {
            let x = j * SEG_LEN + anchor.x;
            let y = anchor.y + i * SEG_LEN;
            let pos = new Vec2D(x, y);
            let curr = i * COLS + j;

            // add vertical constraints
            if (i == 0) {
                scene.add_fixed_constraint(j, pos, SEG_LEN, Cmp.Less);
            } else {
                let below = (i - 1) * COLS + j;
                scene.add_point_constraint(curr, below, SEG_LEN, Cmp.Less);
            }

            // add horizontal constraints
            if (j == COLS - 1) {
                continue;
            }
            scene.add_point_constraint(curr, curr + 1, SEG_LEN, Cmp.Less);
        }
    }
}

function init() {
    let scene = new Scene(600, 600);
    initRope(scene);
    //initCloth(scene);

    const width = scene.width();
    const height = scene.height();

    const canvas = document.getElementById('game-of-life-canvas');
    canvas.width = width;
    canvas.height = height;

    const ctx = canvas.getContext('2d');
    let added = false;
    let ind = null;
    let DIST = 32;
    document.addEventListener('mousemove', (evt) => {
        const rect = canvas.getBoundingClientRect();
        const vec = new Vec2D(evt.clientX - rect.left, evt.clientY - rect.top);
        if (!added) {
            ind = scene.add_particle(vec, vec, 100);
            console.log('ind:', ind);
            added = true;
            for (let i = 0; i < ind + 1; i++) {
                scene.add_point_constraint(i, ind, DIST, Cmp.Greater);
            }
        }

        scene.set_prev_pos(ind, vec);
        scene.set_curr_pos(ind, vec);
    });

    // initialize the timer variables and start the animation
    const renderLoop = (prev) => {
        scene.step();
        ctx.clearRect(0, 0, width, height);
        drawParticles(ctx, scene);

        animationId = requestAnimationFrame(renderLoop);
    };

    playButton = document.querySelector('#toggle-play');
    const play = () => {
        playButton.textContent = '⏸';
        console.log('playing');
        return requestAnimationFrame(renderLoop);
    };

    const pause = () => {
        playButton.textContent = '▶';
        cancelAnimationFrame(animationId);
        animationId = null;
    };

    playButton.addEventListener('click', (evt) => {
        if (animationId === null) {
            play();
            return;
        }

        pause();
    });

    play();
}

function drawParticles(ctx, scene) {
    ctx.strokeStyle = '#000';
    const particlesPtr = scene.particle_positions();
    //TODO DONT USE FIXED NUMBER
    const len = scene.particles_length();
    const particles = new Float32Array(memory.buffer, particlesPtr, len);

    let x = 0;
    let y = 0;
    ctx.beginPath();
    for (let i = 1; i < len; i += 2) {
        x = particles[i - 1];
        y = particles[i];
        ctx.moveTo(x + 4, y);
        ctx.arc(x, y, 4, 0, 2 * Math.PI);
    }
    ctx.stroke();
}
