import {z } from "zod";

export const createMarketSchema = z.object({
    name: z.string(),
    imgURL: z.string(),
    desc: z.string(),
    endDate: z.string().time()
})

export const setMarketOutcome = z.object({
    id: z.string(),
    outcome: z.string(),
})

export const getMarket = z.object({
    id: z.string()
})