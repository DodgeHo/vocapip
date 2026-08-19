use crate::models::{PlayerProgress, ReviewRating};

pub fn award_for_review(rating: ReviewRating, prior_same_day: i64, objective_correct: bool) -> i64 {
    let base = match (rating, prior_same_day) {
        (ReviewRating::Again, 0) => 2,
        (ReviewRating::Hard, 0) => 4,
        (ReviewRating::Good, 0) => 7,
        (ReviewRating::Easy, 0) => 8,
        (ReviewRating::Again | ReviewRating::Hard, 1) => 1,
        (ReviewRating::Good, 1) => 2,
        (ReviewRating::Easy, 1) => 0,
        _ => 0,
    };
    let objective_bonus = i64::from(objective_correct && prior_same_day == 0) * 3;
    base + objective_bonus
}

pub fn level_from_xp(total_xp: i64, streak_days: i64) -> PlayerProgress {
    let xp = total_xp.max(0);
    let mut level = 1_i64;
    let mut consumed = 0_i64;
    loop {
        let required = xp_for_level(level);
        if consumed + required > xp {
            return PlayerProgress {
                xp,
                level,
                xp_into_level: xp - consumed,
                xp_for_next_level: required,
                streak_days: streak_days.max(0),
            };
        }
        consumed += required;
        level += 1;
    }
}

pub fn xp_for_level(current_level: i64) -> i64 {
    60 + 20 * (current_level.max(1) - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratings_follow_product_xp_table() {
        assert_eq!(award_for_review(ReviewRating::Again, 0, false), 2);
        assert_eq!(award_for_review(ReviewRating::Hard, 0, false), 4);
        assert_eq!(award_for_review(ReviewRating::Good, 0, false), 7);
        assert_eq!(award_for_review(ReviewRating::Easy, 0, false), 8);
    }

    #[test]
    fn repeated_easy_cannot_farm_xp() {
        assert_eq!(award_for_review(ReviewRating::Easy, 1, false), 0);
        assert_eq!(award_for_review(ReviewRating::Easy, 8, true), 0);
    }

    #[test]
    fn objective_bonus_is_only_awarded_once() {
        assert_eq!(award_for_review(ReviewRating::Good, 0, true), 10);
        assert_eq!(award_for_review(ReviewRating::Good, 1, true), 2);
    }

    #[test]
    fn level_curve_is_cumulative() {
        assert_eq!(level_from_xp(0, 0).level, 1);
        assert_eq!(level_from_xp(59, 0).level, 1);
        assert_eq!(level_from_xp(60, 0).level, 2);
        let progress = level_from_xp(140, 3);
        assert_eq!(progress.level, 3);
        assert_eq!(progress.xp_into_level, 0);
        assert_eq!(progress.xp_for_next_level, 100);
        assert_eq!(progress.streak_days, 3);
    }
}
