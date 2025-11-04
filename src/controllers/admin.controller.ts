import type{Request, Response} from "express"
import { signUpSchema } from "../types/user.types"
import { Prisma } from "@prisma/client";
import bcrypt from "bcryptjs";
import jwt from "jsonwebtoken"

const JWT_SECRET_TOKEN =  process.env.JWT_SECRET_TOKEN ?? "superSecretePasswords"

export const SignUpAdmin = async(req:Request, res: Response)=> {

    const {success, data } = signUpSchema.safeParse(req.body);

    if(!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, please try again"
        }
        )
    }


    const existingUser  = await prisma.admin.findOne({
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

    const user = prisma.admin.create({
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

export const signInAdmin = async(req: Request, res: Response) =>{
    const {success, data } = signUpSchema.safeParse(req.body);

    if(!success || !data){
        return res.status(411).json({
            message: "Invalid inputs, please try again"
        }
        )
    }

    

    const existingUser  = await prisma.admin.findOne({
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

