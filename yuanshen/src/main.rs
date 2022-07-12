use aopt::err::Error;
use aopt::err::Result;
use aopt::prelude::*;
use aopt::SingleApp;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait State {
    fn get_state(&self) -> i32;

    fn set_state(&mut self, state: i32);
}

impl State for i32 {
    fn get_state(&self) -> i32 {
        *self
    }

    fn set_state(&mut self, state: i32) {
        *self = state
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Units<T>
where
    T: Default + State + Copy + Clone + PartialEq + Eq,
{
    units: Vec<T>,
    max_state: T,
}

impl<T> Units<T>
where
    T: Default + State + Copy + Clone + PartialEq + Eq,
{
    pub fn new(count: usize, max_state: T) -> Self {
        Self {
            units: vec![T::default(); count],
            max_state,
        }
    }

    pub fn with_units(units: Vec<T>, max_state: T) -> Self {
        Self { max_state, units }
    }

    pub fn set_units(&mut self, index: usize, state: T) -> &mut Self {
        assert!(index < self.units.len());
        assert!(state.get_state() < self.max_state.get_state());
        self.units[index] = state;
        self
    }

    pub fn get_units(&self, index: usize) -> &T {
        &self.units[index]
    }

    pub fn get_state(&self, index: usize) -> i32 {
        self.units[index].get_state()
    }

    pub fn set_state(&mut self, index: usize, state: i32) {
        self.units[index].set_state(state);
    }

    fn generate_next_units_numeric(mut self) -> Self {
        let count = self.units.len();
        let max_state = self.max_state.get_state();

        // add first state
        self.set_state(0, self.get_state(0) + 1);
        for index in 0..count {
            if self.units[index].get_state() >= max_state {
                // add next state if current state >= M
                self.set_state(index, 0);
                if index + 1 < count {
                    self.set_state(index + 1, self.get_state(index + 1) + 1);
                }
            }
        }
        self
    }

    pub fn next_units(&self) -> Self {
        let ret = self.clone();
        ret.generate_next_units_numeric()
    }

    pub fn attack(&mut self, indexs: &[usize]) {
        let max = self.max_state.get_state();

        for index in indexs {
            self.set_state(*index, (self.get_state(*index) + 1) % max);
        }
    }

    pub fn find_attack_index(
        &self,
        other: &Self,
        index_map: &HashMap<usize, Vec<usize>>,
    ) -> Option<usize> {
        let count = self.units.len();
        let max_state = self.max_state.get_state();

        for i in 0..count {
            let indexs = &index_map[&i];
            let mut is_child = true;

            for index in 0..count {
                if indexs.contains(&index) {
                    if ((self.get_state(index) + 1) % max_state) != other.get_state(index) {
                        is_child = false;
                    }
                } else if self.get_state(index) != other.get_state(index) {
                    is_child = false;
                }
            }
            if is_child {
                return Some(i);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }
}

#[derive(Debug, Clone)]
pub struct IdxNode {
    idx: usize,
    next_idx: Vec<Option<usize>>,
}

impl IdxNode {
    pub fn new(idx: usize, state_count: usize) -> Self {
        Self {
            idx,
            next_idx: vec![None; state_count],
        }
    }

    pub fn set_next(&mut self, index: usize, idx: usize) {
        assert!(index < self.next_idx.len());
        self.next_idx[index] = Some(idx);
    }
}

#[derive(Debug)]
pub struct Graphics<T>
where
    T: State + Default + Copy + Clone + PartialEq + Eq + Debug,
{
    graphics: Vec<IdxNode>,
    units_queue: Vec<Units<T>>,
}

impl<T> Graphics<T>
where
    T: State + Default + Copy + Clone + PartialEq + Eq + Debug,
{
    pub fn new(beg: &Units<T>, end: &Units<T>) -> Self {
        assert_eq!(beg.len(), end.len());

        let count = beg.len();
        let max_state = beg.get_state(0).get_state();
        let mut units_queue = Vec::with_capacity(count.pow(max_state as u32));
        let mut units = beg.clone();

        loop {
            if &units == end {
                units_queue.push(units);
                break;
            }
            units_queue.push(units.clone());
            units = units.next_units();
        }
        Self {
            graphics: vec![],
            units_queue,
        }
    }

    pub fn init_link(&mut self, index_map: &HashMap<usize, Vec<usize>>) {
        let count = self.units_queue[0].len();

        self.graphics = self
            .units_queue
            .iter()
            .enumerate()
            .map(|(idx, _)| IdxNode::new(idx, count))
            .collect();
        let count = self.graphics.len();

        // current Units
        for i in 0..count {
            for j in 0..count {
                // don't compare itself
                if j != i {
                    let i_units = self.get_units(self.graphics[i].idx);
                    let j_units = self.get_units(self.graphics[j].idx);

                    if let Some(hit_index) = i_units.find_attack_index(j_units, index_map) {
                        self.graphics[i].set_next(hit_index, j);
                    }
                }
            }
        }
    }

    pub fn find_path(&self, beg: &Units<T>, ends: &[Units<T>]) -> Option<Vec<(Units<T>, usize)>> {
        let mut ret: Vec<(Units<T>, usize)> = vec![];
        let mut beg_index = None;

        for (index, node) in self.graphics.iter().enumerate() {
            let units = self.get_units(node.idx);

            if units == beg {
                beg_index = Some(index);
            }
        }
        if let Some(beg_index) = beg_index {
            let mut paths = vec![vec![(beg_index, 0)]];

            while !paths.is_empty() {
                let mut next_paths = vec![];

                for path in paths.iter() {
                    if let Some((last, _)) = path.last() {
                        let idx = self.graphics[*last].idx;
                        let units = self.get_units(idx);

                        // check if we have reach end Units
                        if ends.contains(units) {
                            for (idx, hit_index) in path.iter() {
                                ret.push((self.get_units(*idx).clone(), *hit_index));
                            }
                            return Some(ret);
                        } else {
                            for (hit_index, next_idx) in
                                self.graphics[*last].next_idx.iter().enumerate()
                            {
                                if let Some(idx) = next_idx {
                                    if !path.iter().any(|v| v.0 == *idx) {
                                        let mut next_path = path.clone();

                                        next_path.last_mut().unwrap().1 = hit_index;
                                        next_path.push((*idx, 0));
                                        next_paths.push(next_path);
                                    }
                                }
                            }
                        }
                    }
                }
                paths = next_paths;
            }
        }
        None
    }

    pub fn get_units(&self, index: usize) -> &Units<T> {
        &self.units_queue[index]
    }

    pub fn get_units_count(&self) -> usize {
        self.units_queue.len()
    }

    pub fn display_graphics(&self) {
        for (idx, node) in self.graphics.iter().enumerate() {
            println!(
                "NODE[{}] = {:?} --> link to {:?}",
                idx, self.units_queue[node.idx], node.next_idx
            );
        }
    }
}

fn main() -> Result<()> {
    let mut app = SingleApp::<SimpleSet, DefaultService, ForwardPolicy>::default();

    getopt_add!(app, "-N=u", "Set number of stone")?;
    getopt_add!(app, "-M=u", "Set kind count of stone")?;
    getopt_add!(app, "-L=a", "Set hit index map data")?;
    getopt_add!(app, "-B=s", "Set begin state")?;
    getopt_add!(app, "-E=a", "Set end state")?;

    app.run(&mut std::env::args().skip(1), |ret, app| {
        if ret {
            let empty_data = vec![];
            let set = app.get_parser().get_set();
            let number = *get_value_of(set, "-N")?.as_uint().unwrap() as usize;
            let max_state = *get_value_of(set, "-M")?.as_uint().unwrap() as i32;
            let link_data: &Vec<String> = get_value_of(set, "-L")?.as_vec().unwrap_or(&empty_data);
            let beg_units = get_value_of(set, "-B")?.as_str().unwrap();
            let end_units: &Vec<String> = get_value_of(set, "-E")?.as_vec().unwrap_or(&empty_data);
            let index_map = generate_index_map(link_data);
            let beg_units = generate_beg_number(beg_units)
                .into_iter()
                .map(|v| v as i32)
                .collect();
            let end_units = generate_end_number(end_units);
            let zero = Units::<i32>::new(number, max_state);
            let end = Units::<i32>::with_units(vec![max_state - 1; number], max_state);
            let mut graphics = Graphics::new(&zero, &end);
            let beg = Units::<i32>::with_units(beg_units, max_state);
            let ends: Vec<Units<i32>> = end_units
                .into_iter()
                .map(|v| Units::<i32>::with_units(v, max_state))
                .collect();

            graphics.init_link(&index_map);
            if let Some(path) = graphics.find_path(&beg, &ends) {
                let count = path.len();

                for (index, (units, hit)) in path.iter().enumerate() {
                    match index {
                        0 => {
                            println!("初始状态 = {:?}", units);
                            println!("打击第 {} 个方块", hit + 1);
                        }
                        _ if index == count - 1 => {
                            println!("结束状态 = {:?}", units);
                        }
                        _ => {
                            println!("当前状态 = {:?}", units);
                            println!("打击第 {} 个方块", hit + 1);
                        }
                    }
                }
            } else {
                println!("NOTHING FOUND!");
            }
        }
        Ok(())
    })?;

    Ok(())
}

fn get_value_of<'a, S: Set>(set: &'a S, name: &str) -> Result<&'a OptValue> {
    set.get_value(name)?
        .ok_or_else(|| Error::raise_error(format!("Can not read value of '{}' from set", name)))
}

fn generate_index_map(link_data: &Vec<String>) -> HashMap<usize, Vec<usize>> {
    let mut ret = HashMap::default();

    for data in link_data {
        let parts: Vec<&str> = data.split(':').collect();
        let (idx1, idx_str) = (parts[0], parts[1]);
        let idx1 = idx1.parse::<usize>().unwrap();

        ret.insert(idx1, generate_number_queue(idx_str));
    }
    ret
}

fn generate_beg_number(data: &str) -> Vec<usize> {
    generate_number_queue(data)
}

fn generate_end_number(data: &Vec<String>) -> Vec<Vec<i32>> {
    let mut ret = vec![];

    for data in data {
        ret.push(
            generate_number_queue(data)
                .into_iter()
                .map(|v| v as i32)
                .collect(),
        );
    }
    ret
}

// split at ','
fn generate_number_queue(data: &str) -> Vec<usize> {
    let mut idxs = vec![];

    if !data.is_empty() {
        for idx_part in data.split(',') {
            if !idx_part.is_empty() {
                idxs.push(idx_part.parse::<usize>().unwrap());
            }
        }
    }

    idxs
}
