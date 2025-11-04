import {Router} from "express"
import { signInUser, SignUpUser } from "../controllers/user.controller"

const router = Router()

router.route("/user/signup").post(SignUpUser)
router.route("user/signin").post(signInUser)


export default router;