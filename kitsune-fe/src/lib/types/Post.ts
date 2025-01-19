import type { User } from './User';

type Post = {
	id: string;
	user: User;
	content: string;
	replyCount: number;
	likeCount: number;
	repostCount: number;
	createdAt: string | Date;
};

export type { Post };
