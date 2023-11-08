-- Add migration script here
create table if not exists user (
    `id` integer not null primary key autoincrement,
    `user_name` varchar(64) not null,
    `email` varchar(64) not null,
    `passwd` char(64) not null,
    -- 00 正常 -- 01 待激活 -- 99 已注销
    `status` char(2) not null,
    `avatar` varchar(128) null,
    -- 其他资料待补充
    `active_time` datetime null,
    `update_time` datetime not null
);

create index idx_user_name on user(`user_name`);

create index idx_user_email on user(`email`);

create index idx_user_active_time on user(`active_time`);

create index idx_user_update_time on user(`update_time`);

create table if not exists product (
    `id` integer not null primary key autoincrement,
    `product_no` varchar(64) not null,
    `product_name` varchar(64) not null,
    `desc` varchar(256) null,
    -- 00 正常 -- 99 已下架
    `status` char(2) not null,
    `update_time` datetime not null
);

create index idx_product_no on product(`product_no`);

create table if not exists user_product (
    `id` integer not null primary key autoincrement,
    `user_id` integer not null,
    `product_id` integer not null,
    -- 订阅类型 00 永久 01 月定
    `type` char(2) not null,
    `sub_count` integer not null,
    `sub_time` datetime not null,
    `exp_time` datetime not null
);

create index idx_user_product_user_id on user_product(`user_id`);

create index idx_user_product_product_id on user_product(`product_id`);