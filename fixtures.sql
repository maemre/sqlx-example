INSERT OR IGNORE INTO bookmark (url, title) VALUES
    ('https://doc.rust-lang.org/book/', 'The Rust Programming Language'),
    ('https://www.sqlite.org/', 'SQLite Home Page'),
    ('https://en.wikipedia.org/wiki/SQL', 'SQL - Wikipedia');

INSERT OR IGNORE INTO tag (name) VALUES
    ('rust'),
    ('programming'),
    ('learning'),
    ('sqlite'),
    ('database'),
    ('sql'),
    ('reference');

INSERT OR IGNORE INTO bookmark_tag (bookmark_id, tag_id) VALUES
    -- The Rust Book: rust, programming, learning
    ((SELECT id FROM bookmark WHERE url = 'https://doc.rust-lang.org/book/'),
     (SELECT id FROM tag WHERE name = 'rust')),
    ((SELECT id FROM bookmark WHERE url = 'https://doc.rust-lang.org/book/'),
     (SELECT id FROM tag WHERE name = 'programming')),
    ((SELECT id FROM bookmark WHERE url = 'https://doc.rust-lang.org/book/'),
     (SELECT id FROM tag WHERE name = 'learning')),
    -- SQLite Home Page: sqlite, database
    ((SELECT id FROM bookmark WHERE url = 'https://www.sqlite.org/'),
     (SELECT id FROM tag WHERE name = 'sqlite')),
    ((SELECT id FROM bookmark WHERE url = 'https://www.sqlite.org/'),
     (SELECT id FROM tag WHERE name = 'database')),
    -- SQL - Wikipedia: sql, database, reference
    ((SELECT id FROM bookmark WHERE url = 'https://en.wikipedia.org/wiki/SQL'),
     (SELECT id FROM tag WHERE name = 'sql')),
    ((SELECT id FROM bookmark WHERE url = 'https://en.wikipedia.org/wiki/SQL'),
     (SELECT id FROM tag WHERE name = 'database')),
    ((SELECT id FROM bookmark WHERE url = 'https://en.wikipedia.org/wiki/SQL'),
     (SELECT id FROM tag WHERE name = 'reference'));
