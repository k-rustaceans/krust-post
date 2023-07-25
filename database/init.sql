CREATE TABLE IF NOT EXISTS member (
	id        SERIAL PRIMARY KEY,
	email     VARCHAR(64) NOT NULL UNIQUE,
	password  varchar(64) NOT NULL,
	create_dt timestamptz DEFAULT NULL,
	delete_dt Timestamptz DEFAULT NULL
)
;

CREATE TABLE IF NOT EXISTS post (
	id         serial PRIMARY KEY,
	member_id  integer     DEFAULT NULL,
	create_dt  timestamptz DEFAULT NULL,
	update_dt  timestamptz DEFAULT NULL,
	is_deleted boolean     DEFAULT FALSE,
	CONSTRAINT fk_member FOREIGN KEY (member_id) REFERENCES member (id)
)
;

CREATE TABLE IF NOT EXISTS post_content (
	id        serial PRIMARY KEY,
	post_id   integer     DEFAULT NULL,
	title     varchar(255) NOT NULL,
	content   text        DEFAULT NULL,
	create_dt timestamptz DEFAULT NULL,
	CONSTRAINT fk_post FOREIGN KEY (post_id) REFERENCES post (id)
)
;

CREATE TABLE IF NOT EXISTS post_like (
	member_id integer DEFAULT NULL,
	post_id   integer DEFAULT NULL,
	CONSTRAINT fk_member FOREIGN KEY (member_id) REFERENCES member (id),
	CONSTRAINT fk_post FOREIGN KEY (post_id) REFERENCES post (id)
)
;

CREATE TABLE IF NOT EXISTS comment (
	id        serial PRIMARY KEY,
	post_id   integer     DEFAULT NULL,
	member_id integer     DEFAULT NULL,
	content   text        DEFAULT NULL,
	create_dt timestamptz DEFAULT NULL,
	update_dt timestamptz DEFAULT NULL,
	CONSTRAINT fk_post FOREIGN KEY (post_id) REFERENCES post (id),
	CONSTRAINT fk_member FOREIGN KEY (member_id) REFERENCES member (id)
)
;

CREATE TABLE IF NOT EXISTS comment_like (
	member_id  integer DEFAULT NULL,
	comment_id integer DEFAULT NULL,
	CONSTRAINT fk_member FOREIGN KEY (member_id) REFERENCES member (id),
	CONSTRAINT fk_comment FOREIGN KEY (comment_id) REFERENCES comment (id)
)
;

INSERT INTO
	member(email, password)
VALUES
	('jerok.kim@gmail.com', 'jerok')
;

-- TODO: insert post, post_content, post_like, comment, comment_like
