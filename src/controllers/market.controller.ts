import type{Request, Response} from "express";
import { createMarketSchema, setMarketOutcome } from "../types/market.types";

export const createOrder = async(req: Request, res: Response)=>{
    const {success, data} = createMarketSchema.safeParse(req.body);

    if(!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, try again"
        })
    }

    try{
    const market = await prisma.market.create({
        data: {
            name: data.name,
            desc: data.desc,
            imgURL: data.imgURL,
            endDate: data.endDate,
        }
    })

    if(!market){
        return res.status(400).json({
            message: "Failed to create market"
        })
    }

    return res.status(201).json({
        message: "Market created successfully",
        data: market,
    })

    }catch(err){
    return res.status(400).json({
        message: "Internal server error" + err
    })
}
}


export const updateMarketResult = async(req: Request, res: Response)=>{
    const {success, data} = setMarketOutcome.safeParse(req.body);

    if (!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, try again"
        })
    }

    try{
    const market = await prisma.market.findOne({
        where: {
            id: data.id
        }
    })

    if(!market){
        return res.status(400).json({
            message: "Market not found"
        })
    }

    const updatedMarket = await prisma.market.update({
        where: {
            id: data.id
        },
        data: {
            finalOutcome: data.outcome
        }
    })
}catch(err){
    return res.status(400).json({
        message: "Internal server error" + err
    })
}
}


export const getAllMarket = async(req: Request, res: Response)=> {
    try{
        const allMarkets = await prisma.market.findMany()

        if(!allMarkets){
            return res.status(400).json({
                message : "No markets found"
            })
        }

        return res.status(200).json({
            message: "Markets fetched succssfully",
            data: allMarkets
        })
    }catch(err){
        return res.status(400).json({
            message: "Internal server error" + err
        })
    }
}