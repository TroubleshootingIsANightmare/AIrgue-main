import Image from "next/image";
import { FormEvent } from "react";
import { MessageArea } from "./argument";

export default function Home() {
  return (
    <div>
      <MessageArea/>
    </div>
  );
}

async function onSubmit(event: FormEvent<HTMLFormElement>) {
  event.preventDefault();

  const formData = new FormData(event.currentTarget);
  const response = await fetch('/api/submit/', {method:'POST', body:formData});


}