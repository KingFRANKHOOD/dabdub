import { Module } from '@nestjs/common';
import { AdminWaitlistController } from './admin-waitlist.controller';
import { WaitlistModule } from '../waitlist/waitlist.module';

@Module({
  imports: [WaitlistModule],
  controllers: [AdminWaitlistController],
})
export class AdminModule {}
