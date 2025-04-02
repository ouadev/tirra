Sync Lifecycle:
---------------

at (logout, exit) {
    fetch
    if (my origin_commit != my local_commit ) AND (my origin_commit == their local_commit) AND 
        push
        set my origin_commit to my local_commit
    else
        display notice.
}

at (login) {
    fetch
    if my local_commit == my origin_commit
        *back up database
        replace db
        set my origin_commit to my local_commit

}