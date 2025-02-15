/**
 * Handle unrecoverable exception
 */
pub fn exception(msg: &str) -> () {
    //TODO: write dump to file
    //TODO: save database ?
    panic!("tirra exception: {}", msg);
}
