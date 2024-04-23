-- 公用user和product
create table if not exists user (
    `id` integer not null primary key autoincrement,
    `user_name` varchar(64) null,
    `email` varchar(64) not null,
    `passwd` char(64) not null,
    -- 00 正常 -- 99 已注销
    `status` char(2) not null,
    `update_time` integer null
);

create index idx_user_name on user(`user_name`);

create index idx_user_email on user(`email`);

create table if not exists product (
    `id` integer not null primary key autoincrement,
    `product` varchar(64) not null,
    `desc` varchar(256) null,
    -- 00 正常 -- 99 已下架
    `status` char(2) not null,
    `update_time` integer null
);

create index idx_product on product(`product`);

create table if not exists user_product (
    `id` integer not null primary key autoincrement,
    `product_id` integer not null,
    `user_id` integer not null,
    `avatar` varchar(256) null,
    -- 00 正常 -- 99 已注销
    `status` char(2) not null,
    `update_time` integer not null
);

create index idx_user_product on user_product(`product_id`, `user_id`);

-- hiqradio
create table if not exists hiqradio_recently (
    `id` integer not null primary key autoincrement,
    `user_id` varchar(64) not null,
    `stationuuid` varchar(40) not null,
    `start_time` integer not null,
    `end_time` integer null
);

create table if not exists hiqradio_fav_group (
    `id` integer not null primary key autoincrement,
    `user_id` varchar(64) not null,
    `create_time` integer not null,
    `name` varchar(255) not null,
    `desc` varchar(1024) null,
    `is_def` integer null
);

create table hiqradio_favorite (
    `id` integer primary key autoincrement,
    `user_id` varchar(64) not null,
    `stationuuid` varchar(40) not null,
    `group_id` integer
);

insert into
    product(`product`, `desc`, `status`, `update_time`)
values
    (
        'hiqradio',
        'hiqradio listen the whole world',
        '00',
        unixepoch(current_timestamp)
    );