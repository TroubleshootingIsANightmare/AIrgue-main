'use server';
import { NextApiHandler, NextApiRequest, NextApiResponse } from "next";

export default async function send(msg: string, id: string) {
  console.log("response received");

  console.log(msg);
  await fetch(`http://127.0.0.1:8080/${id}/${msg}`);
}