import Express from "express"
import userRouter from "./src/routes/user.routes"
import adminRouter from "./src/routes/admin.routes"


const app = Express()
const PORT = process.env.PORT ?? 8000;


app.use(Express.json());

app.use("/api/v1/", userRouter);
app.use("/api/v1/", adminRouter);

app.listen(PORT, ()=>{
    console.log("Server is running on port : " + PORT)
})