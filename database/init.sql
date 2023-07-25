CREATE TABLE IF NOT EXISTS users (
	id        SERIAL PRIMARY KEY,
	user_id   VARCHAR(64) NOT NULL UNIQUE,
	password  varchar(64) NOT NULL,
	create_dt TIMESTAMPTZ DEFAULT NULL,
	delete_dt TIMESTAMPTZ DEFAULT NULL
)
;

CREATE TABLE IF NOT EXISTS posts (
	id        SERIAL PRIMARY KEY,
	user_id   VARCHAR(64) DEFAULT NULL,
	create_dt TIMESTAMPTZ DEFAULT NULL,
	update_dt TIMESTAMPTZ DEFAULT NULL,
	state     VARCHAR(32) DEFAULT NULL,
	CONSTRAINT fk_users FOREIGN KEY (user_id) REFERENCES users (user_id)
)
;

CREATE TABLE IF NOT EXISTS post_contents (
	id        SERIAL PRIMARY KEY,
	post_id   INTEGER     DEFAULT NULL,
	title     VARCHAR(255) NOT NULL,
	content   TEXT        DEFAULT NULL,
	create_dt TIMESTAMPTZ DEFAULT NULL,
	CONSTRAINT fk_posts FOREIGN KEY (post_id) REFERENCES posts (id)
)
;

CREATE TABLE IF NOT EXISTS post_likes (
	user_id INTEGER DEFAULT NULL,
	post_id INTEGER DEFAULT NULL,
	CONSTRAINT fk_users FOREIGN KEY (user_id) REFERENCES users (user_id),
	CONSTRAINT fk_posts FOREIGN KEY (post_id) REFERENCES posts (id)
)
;

CREATE TABLE IF NOT EXISTS comments (
	id        SERIAL PRIMARY KEY,
	post_id   INTEGER     DEFAULT NULL,
	user_id   INTERVAL    DEFAULT NULL,
	content   TEXT        DEFAULT NULL,
	create_dt TIMESTAMPTZ DEFAULT NULL,
	update_dt TIMESTAMPTZ DEFAULT NULL,
	CONSTRAINT fk_posts FOREIGN KEY (post_id) REFERENCES posts (id),
	CONSTRAINT fk_users FOREIGN KEY (user_id) REFERENCES users (user_id)
)
;

CREATE TABLE IF NOT EXISTS comment_like (
	user_id    INTEGER DEFAULT NULL,
	comment_id INTEGER DEFAULT NULL,
	CONSTRAINT fk_users FOREIGN KEY (user_id) REFERENCES users (user_id),
	CONSTRAINT fk_comments FOREIGN KEY (comment_id) REFERENCES comments (id)
)
;

INSERT INTO
	users(user_id, password)
VALUES
	('jerok.kim@gmail.com', 'jerok'),
	('krustacean@gmail.com', 'krustacean'),
	('ferris@gmail.com', 'ferris')
;

INSERT INTO
	posts(user_id, create_dt, update_dt, state)
VALUES
	('jerok.kim@gmail.com', NOW(), NOW(), 'Created'),
	('krustacean@gmail.com', NOW(), NOW(), 'Created'),
	('ferris@gmail.com', NOW(), NOW(), 'Created')
;

INSERT INTO
	post_contents(post_id, title, content, create_dt)
VALUES
	(1, 'post title by jerok', 'post content by jerok', NOW()),
	(2, 'post title by krustacean', 'post content by krustacean', NOW()),
	(3, 'post title by ferris', 'post content by ferris', NOW())
;

INSERT INTO
	post_likes(user_id, post_id)
VALUES
	(1, 2),
	(1, 3),
	(3, 2)
;

INSERT INTO
	comments(post_id, user_id, content, create_dt, update_dt)
VALUES
	(1, 2, 'comment by krustacean at post of jerok', NOW(), NOW()),
	(2, 3, 'comment by ferris at post of krustacean', NOW(), NOW()),
	(3, 1, 'comment by jerok at post of ferris', NOW(), NOW())
;

INSERT INTO
	comment_like(user_id, comment_id)
VALUES
	(1, 2),
	(2, 2),
	(3, 2)
;
