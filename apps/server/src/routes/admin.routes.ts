import {Router} from "express"
import { signInAdmin, SignUpAdmin } from "../controllers/admin.controller"


const router = Router()

router.route("/user/signup").post(SignUpAdmin)
router.route("user/signin").post(signInAdmin)


export default router;
