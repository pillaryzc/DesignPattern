pub fn find_champion(grid: Vec<Vec<i32>>) -> i32 {
    grid.iter()
        .enumerate()
        .find(|(i, e)| e.iter().enumerate().all(|(j, v)| j == *i || *v == 1))
        .unwrap()
        .0 as i32
}

pub fn find_champion_II(n: i32, edges: Vec<Vec<i32>>) -> i32 {
    if edges.is_empty() {
        return -1;
    }
    let mut visited = vec![false; n as usize];
    let mut ans = -1;
    edges
        .iter()
        .for_each(|v| visited[*v.get(1).unwrap() as usize] = true);
    visited.iter().enumerate().for_each(|(i, v)| {
        if !v {
            if ans != -1 {
                ans = -1;
                return;
            }
            ans = i as i32;
        }
    });
    ans
}


pub fn maximum_score_after_operations(edges: Vec<Vec<i32>>, values: Vec<i32>) -> i64 {
    todo!()
}
