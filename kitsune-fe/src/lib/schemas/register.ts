import { z } from 'zod';

const registerSchema = z
	.object({
		username: z.string().min(1),
		email: z.string().email(),
		password: z.string().min(1),
		'confirm-password': z.string().min(1)
	})
	.refine((data) => data.password === data['confirm-password'], {
		message: 'Passwords do not match',
		path: ['confirm-password']
	});

export { registerSchema };
