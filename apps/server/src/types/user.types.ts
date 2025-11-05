import {z} from "zod"

export const signUpSchema = z.object({
    email: z.email(),
    password: z.string().min(3).max(12),
    name: z.string(),
})

export const signInSchema  = z.object({
    email: z.email(),
    password: z.string()
})

export const splitSchema = z.object({
    userId: z.string(),
    marketId: z.string(),
    amount: z.number().min(1)
})

export const mergeSchema = z.object({
    userId: z.string(),
    marketId: z.string(),
})