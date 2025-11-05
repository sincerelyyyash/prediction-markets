import type{Request, Response} from "express"
import { signUpSchema, splitSchema } from "../types/user.types"
import { Prisma } from "@prisma/client";
import bcrypt from "bcryptjs";
import jwt from "jsonwebtoken"

const JWT_SECRET_TOKEN =  process.env.JWT_SECRET_TOKEN ?? "superSecretePasswords"

export const SignUpUser = async(req:Request, res: Response)=> {

    const {success, data } = signUpSchema.safeParse(req.body);

    if(!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, please try again"
        }
        )
    }


    const existingUser  = await prisma.user.findOne({
        where: {
            email: data.email
        }
    })

    if(existingUser){
        return res.status(400).json({
            message: "User already exists"
        })
    }


    const hashedPassword = bcrypt.hash(data.password, 10)

    const user = prisma.user.create({
        data: {
            email : data.email,
            password: data.password
        }
    })

    if(!user){
        return res.status(400).json({
            message: "Failed to create user"
        })
    }

    return res.status(201).json({
        message: "user created successfully",
        data: user
    })
}

export const signInUser = async(req: Request, res: Response) =>{
    const {success, data } = signUpSchema.safeParse(req.body);

    if(!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, please try again"
        }
        )
    }

    

    const existingUser  = await prisma.user.findOne({
        where: {
            email: data.email
        }
    })

    const isPasswordValid = bcrypt.compare(data.password, existingUser.password)

    if(!isPasswordValid){
        return res.status(411).json({
            message: "Invalid password"
        })
    }

    const token = jwt.sign(existingUser.id, JWT_SECRET_TOKEN)

    return res.status(200).json({
        message: "User logged in successfully",
        data: token
    })
}


export const split = async(req:Request, res: Response)=> {
    const {success, data } = splitSchema.safeParse(req.body);

    if(!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, try again"
        })
    }

    try{

        const existingMarket = await prisma.market.findOne({
            where: {
                id: data.marketId
            }
        })

        if(!existingMarket){
            return res.status(400).json({
                message: "Market does not exist"
            })
        }

        const user = await prisma.user.findOne({
            where: {
                id: data.userId,
            }
        })

         if(!user){
            return res.status(400).json({
                message: "User does not exist"
            })
        }

        const userBalance = user.balance;

        if(userBalance < data.amount){
            return res.status(400).json({
                message: "Insufficient balance"
            })
        }

        const split = await prisma.$transaction(async (tx) => {

            
            const reduceBalance = await tx.user.update({
                where: {
                    id: data.userId                
                },
                data: {
                    balance: userBalance - data.amount
                }
            })

            const Position = await tx.position.create({
                data: {
                    userId: data.userId,
                    market: data.marketId,
                    YesHolding: data.amount,
                    NoHolding: data.amount,
                }
            })
        })

        if(!split){
            return res.status(400).json({
                message: "Failed to split Market"
            })
        }

        return res.status(201).json({
            message: "Spilt order successful",
        })
  


    }catch(err){
        return res.status(500).json({
            message: "Internal server error" + err
        })
    }
}


export const merge = async(req:Request, res: Response)=>{
    const {success, data} = mergeSchema.safeParse(req.body);
    
    if(!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, try again"
        })
    }

    try{

        const userPosition = await prisma.position.findOne({
            where: {
                userId: data.userId,
                marketId: data.marketId
            }
        })
        
        if(!userPosition){
            return res.status(400).json({
                message: "User does not have positon in this market"
            })
        }

        const mergeOrder = await prisma.$transaction(async (tx) => {

            const YesHolding = userPosition.YesHolding
            const NoHolding = userPosition.NoHolding

            if (YesHolding === NoHolding){

                const user = await tx.user.findOne({
                    where: {
                        id: data.userId
                    }
                })

                const addBalance = await tx.user.update({
                    where:{
                        id: data.userId
                    },
                    data: {
                        balance: user.balance - YesHolding
                    }
                })  


                const reducePosition = await tx.position.findAndupdate({
                    where: {
                        marketId: data.marketId,
                        userId: data.userId
                    },
                    data:{
                        YesHolding : YesHolding - YesHolding,
                        NoHolding: NoHolding - NoHolding,
                    }
                })
            }


            if(YesHolding !== NoHolding){
                
            }
        })




    }catch(err){
        return res.status(500).json({
            message: "Internal server error" +err
        })
    }
}
