type FullUser = {
	description?: string;
	headerUrl?: string;
	createdAt: Date | string;
} & User;

type User = {
	id: string;
	name: string;
	username: string;
	avatarUrl?: string;
};

export type { FullUser, User };
