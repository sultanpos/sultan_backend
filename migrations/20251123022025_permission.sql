-- Add migration script here
CREATE TABLE permissions (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	user_id INTEGER NOT NULL,
    branch_id INTEGER,
    permission INTEGER NOT NULL,
    action INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE, 
    FOREIGN KEY (branch_id) REFERENCES branches(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_permissions_unique_with_branch 
    ON permissions (user_id, permission, branch_id) 
    WHERE branch_id IS NOT NULL;

CREATE UNIQUE INDEX idx_permissions_unique_without_branch 
    ON permissions (user_id, permission) 
    WHERE branch_id IS NULL;

CREATE INDEX idx_permissions_user_id ON permissions (user_id);