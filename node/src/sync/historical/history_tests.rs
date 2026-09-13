use super::scheduler::{PeerHealthConfig, SchedulerConfig};
use super::*;
use eyre::ensure;

fn scheduler(blocks: Vec<u64>) -> PeerWorkScheduler {
    let config = SchedulerConfig {
        max_attempts_per_block: 0,
        ..Default::default()
    };
    let health = Arc::new(PeerHealthTracker::new(
        PeerHealthConfig::from_scheduler_config(&config),
    ));
    PeerWorkScheduler::new_with_health(config, blocks, health)
}

#[tokio::test]
async fn pruned_peer_can_fetch_recent_work_without_consuming_older_work() -> Result<()> {
    let scheduler = scheduler(vec![1, 2, 100, 101]);
    let pruned = PeerId::random();
    ensure!(
        scheduler
            .next_batch_for_peer(pruned, 100..=101)
            .await
            .blocks
            == vec![100, 101],
        "old queue prefix blocked recent history"
    );
    ensure!(
        scheduler.oldest_waiting_block().await == Some(1),
        "older history was lost"
    );
    ensure!(
        scheduler
            .next_batch_for_peer(pruned, 100..=101)
            .await
            .blocks
            .is_empty(),
        "pruned peer received old history"
    );
    ensure!(
        scheduler
            .next_batch_for_peer(PeerId::random(), 0..=101)
            .await
            .blocks
            == vec![1, 2],
        "archive peer could not retrieve retained work"
    );
    Ok(())
}

#[tokio::test]
async fn escalation_obeys_both_history_bounds() -> Result<()> {
    let scheduler = scheduler(vec![1, 100]);
    let peer = PeerId::random();
    ensure!(
        scheduler.next_batch_for_peer(peer, 0..=100).await.blocks == vec![1],
        "initial assignment missing"
    );
    scheduler.requeue_failed(&[1]).await;
    let recent = scheduler.next_batch_for_peer(peer, 100..=100).await;
    ensure!(
        recent.blocks == vec![100] && recent.mode == FetchMode::Normal,
        "out-of-range escalation blocked normal work"
    );
    scheduler.requeue_failed(&[100]).await;
    ensure!(
        scheduler
            .next_batch_for_peer(peer, 2..=99)
            .await
            .blocks
            .is_empty(),
        "escalation escaped history bounds"
    );
    ensure!(
        scheduler.escalation_len().await == 2,
        "unavailable retries were consumed"
    );
    ensure!(
        scheduler.next_batch_for_peer(peer, 0..=99).await.blocks == vec![1],
        "old escalation disappeared"
    );
    ensure!(
        scheduler.next_batch_for_peer(peer, 100..=100).await.blocks == vec![100],
        "recent escalation disappeared"
    );
    Ok(())
}
