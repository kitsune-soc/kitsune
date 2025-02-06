import type { Visibility$options } from '$houdini';

import type { User } from './User';

type Post = {
	id: string;
	user: User;
	content: string;
	replyCount: number;
	likeCount: number;
	repostCount: number;
	url: string;
	createdAt: string | Date;
	visibility: Visibility$options;
};

export type { Post };
