use std::collections::{BTreeSet, HashMap};

pub const FIELD_WIDTH: i32 = 39;
pub const FIELD_HEIGHT: i32 = 20;
pub const MAX_WAVE: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => Self::Up,
            1 => Self::Right,
            2 => Self::Down,
            _ => Self::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn moved(self, direction: Direction) -> Self {
        let (dx, dy) = direction.delta();
        Self::new(self.x + dx, self.y + dy)
    }

    fn in_bounds(self) -> bool {
        self.x >= 1 && self.x < FIELD_WIDTH - 1 && self.y >= 1 && self.y < FIELD_HEIGHT - 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Running,
    Paused,
    Victory,
    Defeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputAction {
    Move(Direction),
    Fire,
    Pause,
    Quit,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallKind {
    Brick,
    Steel,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tank {
    pub pos: Position,
    pub dir: Direction,
    pub move_cooldown: u16,
    pub fire_cooldown: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bullet {
    pub pos: Position,
    pub dir: Direction,
    pub friendly: bool,
}

pub struct Game {
    pub player: Tank,
    pub enemies: Vec<Tank>,
    pub bullets: Vec<Bullet>,
    pub walls: HashMap<Position, WallKind>,
    pub base: Position,
    pub score: u32,
    pub lives: u32,
    pub wave: u32,
    pub status: GameStatus,
    pub remaining_enemies: u32,
    tick_count: u32,
    rng: u64,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self {
            player: Tank {
                pos: Position::new(5, FIELD_HEIGHT - 3),
                dir: Direction::Up,
                move_cooldown: 0,
                fire_cooldown: 0,
            },
            enemies: Vec::new(),
            bullets: Vec::new(),
            walls: HashMap::new(),
            base: Position::new(FIELD_WIDTH - 6, FIELD_HEIGHT - 3),
            score: 0,
            lives: 3,
            wave: 1,
            status: GameStatus::Running,
            remaining_enemies: 0,
            tick_count: 0,
            rng: 0x1a2b3c4d5e6f7788,
        };
        game.start_wave(1);
        game
    }

    pub fn tick(&mut self, action: InputAction) {
        match action {
            InputAction::Pause => {
                self.status = if self.status == GameStatus::Paused {
                    GameStatus::Running
                } else {
                    GameStatus::Paused
                };
                return;
            }
            InputAction::Quit => return,
            _ => {}
        }
        if self.status != GameStatus::Running {
            return;
        }
        self.tick_count = self.tick_count.wrapping_add(1);
        self.handle_player(action);
        self.update_enemies();
        self.update_bullets();
        self.check_wave_complete();
    }

    fn start_wave(&mut self, wave: u32) {
        self.wave = wave;
        self.status = GameStatus::Running;
        self.enemies.clear();
        self.bullets.clear();
        self.remaining_enemies = 6 + wave * 2;
        self.walls = build_walls(wave);
        self.player.pos = Position::new(5, FIELD_HEIGHT - 3);
        self.player.dir = Direction::Up;
        self.player.move_cooldown = 0;
        self.player.fire_cooldown = 0;

        for _ in 0..3 {
            if self.remaining_enemies > 0 {
                self.spawn_enemy();
            }
        }
    }

    fn handle_player(&mut self, action: InputAction) {
        if self.player.move_cooldown > 0 {
            self.player.move_cooldown -= 1;
        }
        if self.player.fire_cooldown > 0 {
            self.player.fire_cooldown -= 1;
        }

        match action {
            InputAction::Move(direction) => {
                self.player.dir = direction;
                if self.player.move_cooldown == 0 && self.can_move_tank(self.player.pos, direction)
                {
                    self.player.pos = self.player.pos.moved(direction);
                    self.player.move_cooldown = 2;
                }
            }
            InputAction::Fire
                if self.player.fire_cooldown == 0
                    && self.bullets.iter().filter(|bullet| bullet.friendly).count() < 2 =>
            {
                self.bullets.push(Bullet {
                    pos: self.player.pos.moved(self.player.dir),
                    dir: self.player.dir,
                    friendly: true,
                });
                self.player.fire_cooldown = 5;
            }
            _ => {}
        }
    }

    fn update_enemies(&mut self) {
        if self.remaining_enemies > 0
            && self.tick_count.is_multiple_of(18)
            && self.enemies.len() < 4
        {
            self.spawn_enemy();
        }

        let mut fired_bullets = Vec::new();
        for index in 0..self.enemies.len() {
            let enemy_speed = (5 - self.wave).max(2) as u16;
            let mut position = self.enemies[index].pos;
            let mut direction = self.enemies[index].dir;
            let mut move_cooldown = self.enemies[index].move_cooldown;
            let mut fire_cooldown = self.enemies[index].fire_cooldown;

            move_cooldown = move_cooldown.saturating_sub(1);
            fire_cooldown = fire_cooldown.saturating_sub(1);

            if move_cooldown == 0 {
                let should_choose_direction = !self.can_move_tank(position, direction)
                    || self.next_random().is_multiple_of(10);
                if should_choose_direction {
                    direction = self.choose_enemy_direction(position);
                }
                if self.can_move_tank(position, direction) {
                    position = position.moved(direction);
                    move_cooldown = enemy_speed;
                }
            }

            if fire_cooldown == 0
                && self.next_random().is_multiple_of(4)
                && (self.has_line_of_sight(position, self.player.pos)
                    || self.has_line_of_sight(position, self.base))
            {
                fired_bullets.push(Bullet {
                    pos: position.moved(direction),
                    dir: direction,
                    friendly: false,
                });
                fire_cooldown = 18;
            }

            let enemy = &mut self.enemies[index];
            enemy.pos = position;
            enemy.dir = direction;
            enemy.move_cooldown = move_cooldown;
            enemy.fire_cooldown = fire_cooldown;
        }

        self.bullets.extend(fired_bullets);
    }

    fn update_bullets(&mut self) {
        for bullet in self.bullets.iter_mut() {
            bullet.pos = bullet.pos.moved(bullet.dir);
        }

        let mut removed_bullets = BTreeSet::new();
        let mut removed_enemies = BTreeSet::new();
        let mut destroyed_walls = Vec::new();
        let mut player_hit = false;
        let mut base_hit = false;

        for (bullet_index, bullet) in self.bullets.iter().enumerate() {
            if !bullet.pos.in_bounds() {
                removed_bullets.insert(bullet_index);
                continue;
            }

            if let Some(kind) = self.walls.get(&bullet.pos) {
                match kind {
                    WallKind::Brick => {
                        destroyed_walls.push(bullet.pos);
                        removed_bullets.insert(bullet_index);
                        continue;
                    }
                    WallKind::Steel => {
                        removed_bullets.insert(bullet_index);
                        continue;
                    }
                    WallKind::Water => {}
                }
            }

            if bullet.pos == self.base {
                base_hit = true;
                removed_bullets.insert(bullet_index);
                continue;
            }

            if bullet.friendly {
                if let Some(enemy_index) = self
                    .enemies
                    .iter()
                    .position(|enemy| enemy.pos == bullet.pos)
                {
                    removed_enemies.insert(enemy_index);
                    removed_bullets.insert(bullet_index);
                }
            } else if bullet.pos == self.player.pos {
                player_hit = true;
                removed_bullets.insert(bullet_index);
            }
        }

        let killed_enemies = removed_enemies.len();
        for position in destroyed_walls {
            self.walls.remove(&position);
        }
        for index in removed_enemies.into_iter().rev() {
            self.enemies.remove(index);
        }
        for index in removed_bullets.into_iter().rev() {
            self.bullets.remove(index);
        }

        self.score += 100 * killed_enemies as u32;
        if base_hit {
            self.status = GameStatus::Defeat;
        }
        if player_hit {
            self.lives = self.lives.saturating_sub(1);
            if self.lives == 0 {
                self.status = GameStatus::Defeat;
            } else {
                self.reset_player();
            }
        }
    }

    fn check_wave_complete(&mut self) {
        if self.remaining_enemies == 0 && self.enemies.is_empty() {
            if self.wave == MAX_WAVE {
                self.status = GameStatus::Victory;
            } else {
                self.start_wave(self.wave + 1);
            }
        }
    }

    fn can_move_tank(&self, position: Position, direction: Direction) -> bool {
        let next = position.moved(direction);
        if !next.in_bounds() {
            return false;
        }
        if self
            .walls
            .get(&next)
            .map(|kind| !matches!(kind, WallKind::Water))
            .unwrap_or(false)
        {
            return false;
        }
        if next == self.base {
            return false;
        }
        if next == self.player.pos {
            return false;
        }
        if self.enemies.iter().any(|enemy| enemy.pos == next) {
            return false;
        }
        true
    }

    fn choose_enemy_direction(&mut self, position: Position) -> Direction {
        if self.next_random().is_multiple_of(4) {
            return Direction::from_index(self.next_random() as usize);
        }
        let target = if self.next_random().is_multiple_of(2) {
            self.player.pos
        } else {
            self.base
        };
        let dx = target.x - position.x;
        let dy = target.y - position.y;
        if self.next_random().is_multiple_of(2) {
            if dx > 0 {
                Direction::Right
            } else if dx < 0 {
                Direction::Left
            } else if dy > 0 {
                Direction::Down
            } else {
                Direction::Up
            }
        } else if dy > 0 {
            Direction::Down
        } else if dy < 0 {
            Direction::Up
        } else if dx > 0 {
            Direction::Right
        } else {
            Direction::Left
        }
    }

    fn has_line_of_sight(&self, from: Position, to: Position) -> bool {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        if (dx != 0 && dy != 0) || dx.abs().max(dy.abs()) > 10 {
            return false;
        }

        let direction = if dx > 0 {
            Direction::Right
        } else if dx < 0 {
            Direction::Left
        } else if dy > 0 {
            Direction::Down
        } else {
            Direction::Up
        };

        let mut current = from;
        while current != to {
            current = current.moved(direction);
            if self
                .walls
                .get(&current)
                .map(|kind| !matches!(kind, WallKind::Water))
                .unwrap_or(false)
            {
                return false;
            }
        }
        true
    }

    fn reset_player(&mut self) {
        self.player.pos = Position::new(5, FIELD_HEIGHT - 3);
        self.player.dir = Direction::Up;
        self.player.move_cooldown = 0;
        self.player.fire_cooldown = 0;
        self.bullets.retain(|bullet| bullet.friendly);
    }

    fn spawn_enemy(&mut self) {
        let candidates = [
            Position::new(5, 2),
            Position::new(FIELD_WIDTH / 2, 2),
            Position::new(FIELD_WIDTH - 6, 2),
        ];
        let position = candidates
            .iter()
            .find(|position| !self.walls.contains_key(position))
            .copied()
            .unwrap_or(Position::new(5, 2));

        self.enemies.push(Tank {
            pos: position,
            dir: Direction::Down,
            move_cooldown: 0,
            fire_cooldown: 30,
        });
        self.remaining_enemies = self.remaining_enemies.saturating_sub(1);
    }

    fn next_random(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }
}

fn build_walls(wave: u32) -> HashMap<Position, WallKind> {
    let mut walls = HashMap::new();
    let base = Position::new(FIELD_WIDTH - 6, FIELD_HEIGHT - 3);
    let player_spawn = Position::new(5, FIELD_HEIGHT - 3);
    let enemy_spawns = [
        Position::new(5, 2),
        Position::new(FIELD_WIDTH / 2, 2),
        Position::new(FIELD_WIDTH - 6, 2),
    ];
    let reserved = |position: Position| {
        position == base || position == player_spawn || enemy_spawns.contains(&position)
    };

    for x in 9..FIELD_WIDTH - 8 {
        for y in 5..FIELD_HEIGHT - 7 {
            if (x + y + wave as i32) % 7 == 0 {
                let position = Position::new(x, y);
                if !reserved(position) {
                    walls.insert(position, WallKind::Brick);
                }
            }
        }
    }

    for &(x, y) in &[(15, 8), (23, 8), (11, 13), (27, 13)] {
        let position = Position::new(x, y);
        if !reserved(position) {
            walls.insert(position, WallKind::Steel);
        }
    }

    for &(x, y) in &[(17, 7), (18, 7), (17, 11), (18, 11)] {
        let position = Position::new(x, y);
        if !reserved(position) {
            walls.insert(position, WallKind::Water);
        }
    }

    for x in base.x - 2..=base.x + 2 {
        for y in base.y - 1..=base.y + 1 {
            let position = Position::new(x, y);
            if position != base && !reserved(position) {
                walls.insert(position, WallKind::Brick);
            }
        }
    }

    walls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_is_initialized() {
        let game = Game::new();
        assert_eq!(game.status, GameStatus::Running);
        assert_eq!(game.lives, 3);
        assert_eq!(game.wave, 1);
        assert_eq!(game.enemies.len(), 3);
        assert_eq!(game.remaining_enemies, 5);
    }

    #[test]
    fn player_moves_in_requested_direction() {
        let mut game = Game::new();
        let original = game.player.pos;
        game.tick(InputAction::Move(Direction::Right));
        assert_eq!(game.player.pos, original.moved(Direction::Right));
    }

    #[test]
    fn player_can_fire_a_bullet() {
        let mut game = Game::new();
        game.tick(InputAction::Fire);
        assert_eq!(game.bullets.len(), 1);
        assert!(game.bullets[0].friendly);
    }

    #[test]
    fn brick_wall_blocks_and_is_destroyed_by_bullet() {
        let mut game = Game::new();
        let wall_position = Position::new(9, 5);
        game.walls.insert(wall_position, WallKind::Brick);
        game.bullets.push(Bullet {
            pos: Position::new(9, 4),
            dir: Direction::Down,
            friendly: true,
        });
        game.update_bullets();
        assert!(!game.walls.contains_key(&wall_position));
        assert!(game.bullets.is_empty());
    }
}
